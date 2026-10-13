#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;

/// DD-132: a run whose tasks completed but whose integration verification failed is
/// repaired by the coordinator without the operator: it revises a task and the run resumes.
#[tokio::test]
async fn the_coordinator_repairs_a_failed_integration_and_the_run_reaches_review() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].test_command = vec![
        support::fixture().to_string_lossy().into_owned(),
        "test-repaired".into(),
    ];
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.args = vec!["review".into()];
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(&dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(failed.state, "failed");
    assert!(workflow::repairable(&failed));

    let repaired = workflow::repair(&dir, 10, failed).await.unwrap();
    assert_eq!(repaired.state, "waiting_approval", "{:?}", repaired.error);
    assert_eq!(repaired.review_round, 1);
    let one = repaired.tasks.iter().find(|t| t.id == "one.txt").unwrap();
    assert_eq!(one.attempt, 2);
    std::fs::remove_dir_all(root).unwrap();
}

/// Without a recorded coordinator the failure is left for the operator unchanged.
#[tokio::test]
async fn a_run_without_a_recorded_coordinator_keeps_its_failure() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].test_command = vec![
        support::fixture().to_string_lossy().into_owned(),
        "test-repaired".into(),
    ];
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let kept = workflow::repair(&dir, 10, failed).await.unwrap();
    assert_eq!(kept.state, "failed");
    assert_eq!(kept.review_round, 0);
    std::fs::remove_dir_all(root).unwrap();
}
