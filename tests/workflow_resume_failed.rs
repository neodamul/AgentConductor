#![cfg(unix)]
//! GitHub #625 / DD-132: Resume on a failed run retries its failed tasks from the retained work, and runs a failed
//! integration verification again; it no longer ends with "a failed run cannot be resumed".
use agentconductor::workflow;
use std::fs;
#[path = "support/workflow.rs"]
mod support;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tokio::test]
async fn resume_retries_the_failed_tasks_of_a_failed_run() {
    let (root, mut plan) = support::setup("shared");
    let markers = root.join("markers");
    fs::create_dir_all(&markers).unwrap();
    // SAFETY: set before this test starts any thread that reads it; no other test in this binary uses it.
    unsafe { std::env::set_var("MOCK_MARKERS", &markers) };
    plan.agents[0].executable = support::fail_once_fixture().to_string_lossy().into_owned();
    plan.agents[0].args = vec![];
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 20)
        .await
        .unwrap();
    assert_eq!(report.state, "failed", "{:?}", report.error);
    assert!(
        report
            .tasks
            .iter()
            .all(|t| t.status == "failed" || t.status == "blocked")
    );
    let report = workflow::resume_due(&dir, now(), 20).await.unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert!(report.tasks.iter().all(|t| t.status == "completed"));
    assert!(report.error.is_none() && report.review_round == 0);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn resume_runs_a_failed_integration_verification_again() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 20)
        .await
        .unwrap();
    let mut store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    let mut report = store.document("workflow").unwrap();
    report["state"] = "failed".into();
    report["error"] = "integration verification failed: a flaky test".into();
    store.record("workflow", &report, "failed fixture").unwrap();
    drop(store);
    let report = workflow::resume_due(&dir, now(), 20).await.unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert!(report.error.is_none());
    fs::remove_dir_all(root).unwrap();
}

// GitHub #637: a coordinator review that ended blocked without revisions is retried by Resume; a run with nothing
// to resume says which task or state blocks it.
#[tokio::test]
async fn resume_retries_a_blocked_review_and_names_what_blocks_otherwise() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 20)
        .await
        .unwrap();
    let mut store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    let mut report = store.document("workflow").unwrap();
    report["state"] = "blocked".into();
    report["latest_review"] =
        serde_json::json!({"verdict":"blocked","summary":"unclear","evidence":[],"revisions":[]});
    store
        .record("workflow", &report, "blocked review fixture")
        .unwrap();
    drop(store);
    let report = workflow::load(&dir).unwrap();
    assert_eq!(workflow::resume_blocker(&dir, &report).unwrap(), None);
    let report = workflow::resume_due(&dir, now(), 20).await.unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert!(report.latest_review.is_none());
    // Everything completed and the run waits for approval: nothing to resume, and the state is named.
    let blocker = workflow::resume_blocker(&dir, &report).unwrap();
    assert_eq!(blocker, Some((String::new(), "waiting_approval".into())));
    fs::remove_dir_all(root).unwrap();
}
