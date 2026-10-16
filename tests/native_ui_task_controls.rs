use agentconductor::native_ui_probe::room::task_controls::{self, Action};
use agentconductor::workflow;
use eframe::egui::{self, accesskit::Role};
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
                "tasks":[{"id":"tests","agent":"codex","prompt":"Write tests"}],
                "agents":[{"id":"codex","parser":"codex"},{"id":"local","parser":"custom"}]
            },
            "tasks":[{"id":"tests","status":"pending","feedback":null}]
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
fn task_controls_localize_the_task_picker_and_feedback_action() {
    for language in ["en", "ko"] {
        let snapshot = snapshot();
        let mut state = task_controls::State::default();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut first = context.run_ui(Default::default(), |ui| {
            assert_eq!(
                task_controls::show(&mut state, &snapshot, false, language, ui),
                None
            );
        });
        first.textures_delta.set.clear();
        first.textures_delta.free.clear();
        let mut output = context.run_ui(Default::default(), |ui| {
            assert_eq!(
                task_controls::show(&mut state, &snapshot, false, language, ui),
                None
            );
        });
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let labels = nodes
            .iter()
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        let buttons = nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Button)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for key in ["room.task_inspector", "room.send_feedback_823598"] {
            let expected = agentconductor::ui::messages::in_language(language, key);
            assert!(labels.contains(&expected), "missing {key} for {language}");
        }
        assert!(buttons.contains(&agentconductor::ui::messages::in_language(
            language,
            "room.send_feedback_823598"
        )));
    }
}

#[tokio::test]
async fn native_task_actions_reach_the_existing_controller_and_persist() {
    let (root, mut plan) = support::setup("worktree");
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
    assert_eq!(
        report.tasks[0].feedback.as_deref(),
        Some("Cover the missing boundary case")
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
