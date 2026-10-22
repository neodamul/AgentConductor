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

#[tokio::test]
async fn a_failed_task_without_a_recorded_coordinator_stays_for_the_operator() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert!(workflow::repairable(&failed));
    let kept = workflow::repair(&dir, 10, failed).await.unwrap();
    assert_eq!(kept.state, "failed");
    assert_eq!(kept.review_round, 0);
    assert!(kept.tasks.iter().any(|task| task.status == "failed"));
    std::fs::remove_dir_all(root).unwrap();
}

fn coordinator_file(plan: &agentconductor::plan::Plan, dir: &std::path::Path) {
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.args = vec!["review".into()];
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
}

/// #135: a task that exits with an error is triaged by the coordinator, not the operator.
#[tokio::test]
async fn the_coordinator_triages_a_failed_task_and_the_run_reaches_review() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    coordinator_file(&plan, &dir);
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(failed.state, "failed");
    assert!(failed.tasks.iter().any(|t| t.status == "failed"));
    assert!(
        workflow::repairable(&failed),
        "{:?} {:?}",
        failed.error,
        failed
            .tasks
            .iter()
            .map(|t| (&t.status, &t.error))
            .collect::<Vec<_>>()
    );

    let repaired = workflow::repair(&dir, 10, failed).await.unwrap();
    assert_eq!(repaired.state, "waiting_approval", "{:?}", repaired.error);
    assert_eq!(repaired.review_round, 1);
    let one = repaired.tasks.iter().find(|t| t.id == "one.txt").unwrap();
    assert_eq!(one.attempt, 2);
    std::fs::remove_dir_all(root).unwrap();
}

/// A failure that only a person may resolve is never triaged: out-of-scope changes and runs
/// that wait for an account.
#[tokio::test]
async fn scope_failures_and_availability_waits_are_not_triaged() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    let mut report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert!(workflow::repairable(&report));
    report.error = Some("out-of-scope change: x.txt".into());
    assert!(!workflow::repairable(&report));
    report.error = Some("task failed".into());
    assert!(workflow::repairable(&report));
    report.tasks[0].status = "waiting_availability".into();
    assert!(!workflow::repairable(&report));
    std::fs::remove_dir_all(root).unwrap();
}
