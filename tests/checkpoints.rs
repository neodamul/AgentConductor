#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;

#[tokio::test]
async fn quota_resume_refuses_changed_shared_partial_files() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-once".into()];
    plan.agents[0]
        .environment
        .insert("MOCK_MARKERS".into(), root.to_str().unwrap().into());
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    workflow::schedule(&dir, "one.txt", "1970-01-01T00:00:01Z").unwrap();
    std::fs::write(root.join("repo/one.txt"), "unexpected edit\n").unwrap();
    assert!(workflow::resume_due(&dir, i64::MAX, 10).await.is_err());
    assert_eq!(workflow::load(&dir).unwrap().tasks[0].attempt, 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn explicit_resume_refuses_changed_worktree_even_when_target_head_matches() {
    let (root, mut plan) = support::setup("worktree");
    plan.agents[0].args = vec!["quota".into()];
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    workflow::stop(&dir, "review").unwrap();
    std::fs::write(report.tasks[0].cwd.join("one.txt"), "unexpected edit\n").unwrap();
    assert!(workflow::resume(&dir, 10).await.is_err());
    assert_eq!(workflow::load(&dir).unwrap().state, "stopped");
    std::fs::remove_dir_all(root).unwrap();
}
