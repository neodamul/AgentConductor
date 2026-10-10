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
async fn unverifiable_coordinator_plans_in_a_disposable_tracked_files_snapshot() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    std::fs::write(repo.join(".gitignore"), "secret.env\n").unwrap();
    for args in [vec!["add", ".gitignore"], vec!["commit", "-m", "ignore"]] {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(repo.join("secret.env"), "TOKEN=private\n").unwrap();
    let mut profiles = plan.agents.clone();
    profiles[0].args = vec!["plan-hostile".into()];
    assert_eq!(profiles[0].permission_mode, "external_sandbox");
    profiles[0].permission_mode = "unverified".into();
    let result = agentconductor::planner::generate(
        "Create three files",
        &profiles,
        "mock",
        &repo,
        &root.join("planning"),
    )
    .await
    .unwrap();
    assert_eq!(result.tasks.len(), 3);
    assert!(repo.join("base.txt").exists());
    assert!(!repo.join("hostile.txt").exists());
    assert_eq!(
        std::fs::read_to_string(repo.join("secret.env")).unwrap(),
        "TOKEN=private\n"
    );
    assert!(!root.join("planning/snapshot").exists());
    agentconductor::git::Repository::open(&repo)
        .unwrap()
        .clean()
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
