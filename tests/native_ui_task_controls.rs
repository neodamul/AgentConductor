use agentconductor::native_ui_probe::room::task_controls::{self, Action};
use agentconductor::workflow;
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

#[path = "support/workflow.rs"]
mod support;

fn snapshot() -> Value {
    json!({
        "revision":"r1",
        "job":{"state":"completed"},
        "report":{
            "state":"ready",
            "plan":{
        "tasks":[{"id":"tests","agent":"codex","model":null,"prompt":"Write tests"}],
                "agents":[{"id":"codex","parser":"codex","args":[],"catalog":{"models":["codex-small","codex-large"],"source":"observed","queried_at":1,"cli_version":"test"}},{"id":"local","parser":"custom"}]
            },
            "tasks":[{"id":"tests","status":"pending","session_id":"session-tests","feedback":null}]
        }
    })
}

#[test]
fn feedback_and_reassignment_requests_are_revision_bound_and_validate_targets() {
    let snapshot = snapshot();
    let feedback = task_controls::request(
        &snapshot,
        Action::Feedback {
            task: "tests".into(),
            feedback: "Cover a timeout case".into(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&feedback).unwrap(),
        json!({"kind":"action","revision":"r1","action":{"kind":"revise","task":"tests","feedback":"Cover a timeout case"}})
    );

    let reassignment = task_controls::request(
        &snapshot,
        Action::Reassign {
            task: "tests".into(),
            agent: "local".into(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&reassignment).unwrap()["action"],
        json!({"kind":"reassign","task":"tests","agent":"local"})
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::Reassign {
                task: "tests".into(),
                agent: "unknown".into(),
            }
        )
        .is_err()
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::Feedback {
                task: "missing".into(),
                feedback: "Revise".into(),
            }
        )
        .is_err()
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::Feedback {
                task: "tests".into(),
                feedback: "  ".into(),
            }
        )
        .is_err()
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::Feedback {
                task: "tests".into(),
                feedback: "x".repeat(16 * 1024 + 1),
            }
        )
        .is_err()
    );
}

#[test]
fn model_changes_are_revision_bound_and_require_an_observed_model() {
    let snapshot = snapshot();
    let bytes = task_controls::request(
        &snapshot,
        Action::SetModel {
            task: "tests".into(),
            model: "codex-large".into(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&bytes).unwrap(),
        json!({"kind":"action","revision":"r1","action":{"kind":"model","task":"tests","model":"codex-large"}})
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::SetModel {
                task: "tests".into(),
                model: "unobserved-model".into(),
            }
        )
        .is_err()
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::SetModel {
                task: "tests".into(),
                model: String::new(),
            }
        )
        .is_err()
    );
    let mut active = snapshot.clone();
    active["report"]["state"] = json!("running");
    assert!(
        task_controls::request(
            &active,
            Action::SetModel {
                task: "tests".into(),
                model: "codex-large".into(),
            }
        )
        .is_err()
    );
    let mut active_task = snapshot.clone();
    active_task["report"]["tasks"][0]["status"] = json!("running");
    assert!(
        task_controls::request(
            &active_task,
            Action::SetModel {
                task: "tests".into(),
                model: "codex-large".into(),
            }
        )
        .is_err()
    );
}

#[tokio::test]
async fn native_task_actions_reach_the_existing_controller_and_persist() {
    let (root, mut plan) = support::setup("worktree");
    plan.agents[0].args.push("{model}".into());
    plan.agents[0].catalog = Some(agentconductor::plan::ModelCatalog {
        models: vec!["test-model".into()],
        source: "test".into(),
        queried_at: 1,
        cli_version: "test".into(),
    });
    let mut backup = plan.agents[0].clone();
    backup.id = "backup".into();
    plan.agents.push(backup);
    let run = root.join("run");
    workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    let room =
        agentconductor::ui::Room::open(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor")))
            .unwrap();
    let mut snapshot = room
        .dispatch(br#"{"kind":"snapshot","room":true}"#)
        .unwrap();
    for action in [
        Action::Reassign {
            task: "one.txt".into(),
            agent: "backup".into(),
        },
        Action::SetModel {
            task: "one.txt".into(),
            model: "test-model".into(),
        },
        Action::Feedback {
            task: "one.txt".into(),
            feedback: "Cover the missing boundary case".into(),
        },
    ] {
        let request = task_controls::request(&snapshot, action).unwrap();
        assert_eq!(room.dispatch(&request).unwrap()["accepted"], true);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let status = room
                .dispatch(br#"{"kind":"snapshot","room":true}"#)
                .unwrap();
            snapshot = status.clone();
            if status["job"]["state"] != "running" {
                assert_ne!(status["job"]["state"], "failed", "{:?}", status["job"]);
                break;
            }
            assert!(std::time::Instant::now() < deadline, "action timed out");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let report = workflow::load(&run).unwrap();
    assert_eq!(report.plan.tasks[0].agent, "backup");
    assert_eq!(report.plan.tasks[0].model.as_deref(), Some("test-model"));
    assert_eq!(
        report.tasks[0].feedback.as_deref(),
        Some("Cover the missing boundary case")
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
