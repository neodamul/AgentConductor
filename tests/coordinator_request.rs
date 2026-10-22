//! GitHub #165 / R-171 / DD-181: feedback sent to the coordinator reaches its workflow, is acted
//! on without restarting finished work, and every state is listed.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow::{self, decisions};

const ASK: &str = "Verification passed but the run shows a failure; reconcile it.";

/// A run whose first task failed, with the amend fixture recorded as the coordinator.
async fn failed_run(
    mode: &str,
    with_coordinator: bool,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let (root, mut plan) = support::setup("shared");
    let root = root.canonicalize().unwrap();
    plan.tasks[0].prompt = "fail".into();
    let run = root.join("run");
    if with_coordinator {
        let mut coordinator = plan.agents[0].clone();
        coordinator.id = "coordinator".into();
        coordinator.permission_mode = "read_only".into();
        coordinator.executable = support::amend_fixture().to_string_lossy().into_owned();
        coordinator.args = vec![mode.into()];
        std::fs::write(
            agentconductor::native_guard::orchestrator_path(&run),
            serde_json::to_vec(&coordinator).unwrap(),
        )
        .unwrap();
    }
    let failed = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(failed.state, "failed");
    (root, run)
}

fn request_rows(run: &std::path::Path) -> Vec<(String, String)> {
    decisions::list(run)
        .unwrap()
        .iter()
        .filter(|r| r["kind"] == "request")
        .map(|r| {
            (
                r["judgment"].as_str().unwrap().into(),
                r["detail"].as_str().unwrap().into(),
            )
        })
        .collect()
}

#[tokio::test]
async fn an_answer_is_listed_through_every_state_and_restarts_nothing() {
    let (root, run) = failed_run("answer", true).await;
    let before = workflow::load(&run).unwrap();
    let proposal = workflow::coordinator_request(&run, 10, ASK).await.unwrap();
    assert_eq!(proposal.summary, "Investigated the discrepancy");

    let rows = request_rows(&run);
    let states: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(states, ["queued", "processing", "completed"], "{rows:?}");
    assert!(rows[0].1.contains("reconcile"));
    assert!(
        rows[2].1.contains("Investigated the discrepancy"),
        "{rows:?}"
    );

    let after = workflow::load(&run).unwrap();
    for (b, a) in before.tasks.iter().zip(&after.tasks) {
        assert_eq!(
            a.attempt, b.attempt,
            "{}: no task is restarted by a question",
            a.id
        );
    }
    assert_eq!(after.state, "failed", "an answer does not change the run");
    assert_eq!(after.review_round, before.review_round + 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_revision_restarts_only_the_revised_task_and_resumes_the_run() {
    let (root, run) = failed_run("revise", true).await;
    workflow::coordinator_request(&run, 10, ASK).await.unwrap();
    let after = workflow::load(&run).unwrap();
    assert_eq!(
        after.state, "waiting_approval",
        "{:?} {:?}",
        after.error, after.review_error
    );
    assert_eq!(after.tasks[0].attempt, 2);
    assert_eq!(after.tasks[1].attempt, 1, "finished tasks keep their work");
    let states: Vec<String> = request_rows(&run).into_iter().map(|r| r.0).collect();
    assert_eq!(states.last().map(String::as_str), Some("completed"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_failing_coordinator_is_recorded_and_bad_requests_are_refused() {
    let (root, run) = failed_run("fail", true).await;
    let error = workflow::coordinator_request(&run, 10, ASK)
        .await
        .unwrap_err();
    assert!(!error.is_empty());
    let rows = request_rows(&run);
    assert_eq!(rows.last().unwrap().0, "failed", "{rows:?}");
    assert!(
        workflow::coordinator_request(&run, 10, "   ")
            .await
            .is_err()
    );
    assert!(
        workflow::coordinator_request(&run, 10, &"x".repeat(9000))
            .await
            .is_err()
    );
    assert!(
        workflow::coordinator_request(&run, 10, "a\0b")
            .await
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();

    let (root, run) = failed_run("answer", false).await;
    let error = workflow::coordinator_request(&run, 10, ASK)
        .await
        .unwrap_err();
    assert!(error.contains("coordinator"), "{error}");
    assert!(request_rows(&run).is_empty(), "nothing was queued");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_room_sends_coordinator_feedback_through_its_own_action() {
    let js = include_str!("../src/ui/room.js");
    assert!(js.contains("coordinator_request"));
    assert!(js.contains("COORDINATOR_DRAFT"));
    let enabled = js
        .split("function feedbackEnabled")
        .nth(1)
        .unwrap()
        .split('\n')
        .next()
        .unwrap();
    assert!(!enabled.contains("!coordinatorView &&"), "{enabled}");
    for catalog in [
        include_str!("../src/ui/messages/en.json"),
        include_str!("../src/ui/messages/ko.json"),
    ] {
        for id in [
            "room.judgment_request_queued",
            "room.judgment_request_processing",
            "room.judgment_request_completed",
            "room.judgment_request_failed",
            "room.coordinator_request_sent",
            "room.send_a_request_to_the_coordinator",
        ] {
            assert!(catalog.contains(&format!("\"{id}\"")), "{id}");
        }
    }
    assert!(include_str!("../src/commands/memory_ui.rs").contains("\"coordinator-request\""));
}

/// The room's action runs the broker command and the request ends up in the judgment list.
#[tokio::test]
async fn the_room_action_delivers_the_request_to_the_coordinator() {
    let (root, run) = failed_run("answer", true).await;
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::open(&run, exe).unwrap();
    let ask = |v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    let snapshot = ask(serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(snapshot["coordinator"], "coordinator");
    ask(serde_json::json!({"kind":"action","revision":snapshot["revision"],"action":{"kind":"coordinator_request","message":ASK}})).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        let state = ask(serde_json::json!({"kind":"snapshot"})).unwrap();
        if state["job"]["state"] == "completed" || state["job"]["state"] == "failed" {
            assert_eq!(state["job"]["state"], "completed", "{}", state["job"]);
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the job did not finish"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let states: Vec<String> = request_rows(&run).into_iter().map(|r| r.0).collect();
    assert_eq!(states, ["queued", "processing", "completed"]);
    // The other development tasks did not run again.
    assert!(
        workflow::load(&run)
            .unwrap()
            .tasks
            .iter()
            .skip(1)
            .all(|t| t.attempt == 1)
    );
    std::fs::remove_dir_all(root).unwrap();
}
