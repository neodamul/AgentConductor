#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn coordinator_decomposes_goal_into_three_validated_tasks_using_registry() {
    let (root, plan) = support::setup("shared");
    let mut profiles = plan.agents.clone();
    profiles[0].args = vec!["plan".into()];
    profiles[0].permission_mode = "read_only".into();
    let result = agentconductor::planner::generate(
        "Create three files",
        &profiles,
        "mock",
        &root.join("repo"),
        &root.join("planning"),
    )
    .await
    .unwrap();
    assert_eq!(result.tasks.len(), 3);
    assert_eq!(result.agents[0].executable, profiles[0].executable);
    result.validate().unwrap();
    agentconductor::git::Repository::open(&root.join("repo"))
        .unwrap()
        .clean()
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn planning_without_read_only_capability_is_rejected_before_spawn() {
    let (root, plan) = support::setup("shared");
    assert!(
        agentconductor::planner::generate(
            "Create three files",
            &plan.agents,
            "mock",
            &root.join("repo"),
            &root.join("planning")
        )
        .await
        .is_err()
    );
    assert!(!root.join("planning").exists());
    std::fs::remove_dir_all(root).unwrap();
}
