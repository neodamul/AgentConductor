#[path = "support/workflow.rs"]
mod support;

use agentconductor::workflow;
use std::time::Duration;

async fn wait_until_active(dir: &std::path::Path) {
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            if workflow::load(dir).is_ok_and(|report| report.tasks[0].status == "running") {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
}

fn run_slow(
    dir: &std::path::Path,
    plan: agentconductor::plan::Plan,
) -> tokio::task::JoinHandle<Result<workflow::Report, String>> {
    let repo = dir.join("repo");
    let run_dir = dir.join("run");
    tokio::spawn(async move { workflow::run(plan, &repo, &run_dir, 10).await })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_feedback_is_bounded_queued_and_applied_to_the_next_attempt() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["task-slow".into()];
    let run = run_slow(&root, plan);
    let dir = root.join("run");
    wait_until_active(&dir).await;

    workflow::revise(&dir, "one.txt", "Add the missing boundary case").unwrap();
    workflow::revise(&dir, "one.txt", "Keep the public API unchanged").unwrap();
    assert!(workflow::revise(&dir, "missing", "Change another task").is_err());
    assert!(workflow::revise(&dir, "one.txt", "").is_err());
    assert!(workflow::revise(&dir, "one.txt", &"x".repeat(16 * 1024)).is_err());
    let active = workflow::load(&dir).unwrap();
    assert_eq!(active.tasks[0].attempt, 1);
    assert_eq!(active.tasks[0].status, "running");

    let report = run.await.unwrap().unwrap();
    assert_eq!(report.tasks[0].attempt, 2);
    assert_eq!(report.tasks[0].status, "completed");
    let initial = std::fs::read_to_string(dir.join("task-0/prompt.txt")).unwrap();
    assert!(!initial.contains("Add the missing boundary case"));
    let prompt = std::fs::read_to_string(report.tasks[0].artifacts.join("prompt.txt")).unwrap();
    assert!(prompt.contains("Add the missing boundary case"));
    assert!(prompt.contains("Keep the public API unchanged"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_feedback_survives_explicit_stop_without_restarting_the_worker() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["task-slow".into()];
    let run = run_slow(&root, plan);
    let dir = root.join("run");
    wait_until_active(&dir).await;

    workflow::revise(&dir, "one.txt", "Keep this for the next attempt").unwrap();
    workflow::stop(&dir, "Operator requested stop").unwrap();
    let report = run.await.unwrap().unwrap();
    assert_eq!(report.state, "stopped");
    assert_eq!(report.tasks[0].attempt, 1);
    assert_eq!(report.tasks[0].status, "stopped");
    assert_eq!(
        report.tasks[0].feedback.as_deref(),
        Some("Keep this for the next attempt")
    );
    std::fs::remove_dir_all(root).unwrap();
}
