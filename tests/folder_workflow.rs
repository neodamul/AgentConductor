use agentconductor::{folder, workflow};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_plain_folder_runs_in_the_selected_project_and_commits_after_approval() {
    let (root, plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    std::fs::write(project.join("one.txt"), b"original human file").unwrap();
    let run = root.join("run");
    let report = workflow::run(plan, &project, &run, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert_eq!(report.tests.len(), 3);
    assert_eq!(
        std::fs::read(project.join("one.txt")).unwrap(),
        b"implemented\n"
    );
    assert_eq!(report.repo, project.canonicalize().unwrap());
    assert!(!project.join(".git").exists());
    let store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
    assert_eq!(
        store.document("execution_baseline").unwrap()["external_history"],
        true
    );
    assert!(folder::owned_changes(&store, &report).unwrap().is_empty());
    std::fs::write(project.join("one.txt"), b"later manual edit").unwrap();
    assert!(
        folder::owned_changes(&store, &report)
            .unwrap()
            .contains(&"one.txt".into())
    );
    assert!(workflow::reject(&run, "reject").is_err());
    assert_eq!(
        std::fs::read(project.join("one.txt")).unwrap(),
        b"later manual edit"
    );
    std::fs::write(project.join("one.txt"), b"implemented\n").unwrap();
    drop(store);
    assert!(workflow::merge(&run).is_err());
    let candidate = report.candidate.unwrap();
    workflow::approve(&run, &candidate.commit).unwrap();
    workflow::merge(&run).unwrap();
    for file in ["one.txt", "two.txt", "three.txt"] {
        assert!(project.join(file).is_file());
    }
    assert!(!project.join(".git").exists());
    let completed = workflow::load(&run).unwrap();
    assert_eq!(completed.state, "completed");
    assert!(
        completed.repo.join("one.txt").exists(),
        "selected project must retain completed results"
    );
    assert_eq!(
        agentconductor::git::Repository::open(&completed.repo)
            .unwrap()
            .text(&["cat-file", "-t", &candidate.commit])
            .unwrap(),
        "commit"
    );
    let room = agentconductor::ui::Room::open(
        &run,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    for packet in [
        serde_json::json!({"kind":"snapshot"}),
        serde_json::json!({"kind":"evidence"}),
        serde_json::json!({"kind":"task","index":0}),
    ] {
        room.dispatch(packet.to_string().as_bytes())
            .expect("completed evidence remains available after compaction");
    }
    drop(room);
    std::fs::remove_dir_all(folder::history_directory(&project).unwrap()).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_changed_original_folder_revokes_approval_without_overwriting_it() {
    let (root, plan) = support::setup("worktree");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    let source = folder::prepare(&project, &root.join("private")).unwrap();
    let run = root.join("run");
    let report = workflow::run(plan, &source, &run, 10).await.unwrap();
    let candidate = report.candidate.unwrap();
    workflow::approve(&run, &candidate.commit).unwrap();
    std::fs::write(project.join("base.txt"), b"human changed this").unwrap();
    assert!(
        workflow::merge(&run)
            .unwrap_err()
            .contains("folder changed")
    );
    assert_eq!(
        std::fs::read(project.join("base.txt")).unwrap(),
        b"human changed this"
    );
    assert!(!project.join("one.txt").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshots_exclude_build_outputs_and_secrets_and_refuse_aliases() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    let project = root.join("project");
    std::fs::create_dir_all(project.join("node_modules")).unwrap();
    std::fs::write(project.join(".env"), "secret").unwrap();
    std::fs::write(project.join("node_modules/large"), "excluded").unwrap();
    std::fs::write(project.join("code.txt"), "code").unwrap();
    let workspace = folder::prepare(&project, &root.join("private")).unwrap();
    assert!(workspace.join("code.txt").is_file());
    assert!(!workspace.join(".env").exists());
    assert!(!workspace.join("node_modules").exists());
    let source = folder::load(&workspace).unwrap().unwrap();
    for path in ["../escape", "node_modules/new", ".env", "/absolute"] {
        assert!(source.safe_path(path).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("outside"), project.join("alias")).unwrap();
        assert!(source.validate().is_err());
        assert!(folder::prepare(&project, &root.join("private")).is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn generate_plan_uses_a_private_read_only_snapshot_of_a_plain_folder() {
    let (root, plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    let workspace = folder::prepare(&project, &root.join("private")).unwrap();
    let mut agents = plan.agents.clone();
    agents[0].args = vec!["plan".into()];
    agents[0].permission_mode = "read_only".into();
    let generated = agentconductor::planner::generate(
        "Create three files",
        &agents,
        "mock",
        &workspace,
        &root.join("plan"),
    )
    .await
    .unwrap();
    assert_eq!(generated.tasks.len(), 3);
    assert!(!project.join(".git").exists());
    assert_eq!(std::fs::read(project.join("base.txt")).unwrap(), b"base\n");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn applying_a_folder_candidate_preserves_backups_for_modified_and_deleted_files() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("changed.txt"), "old").unwrap();
    std::fs::write(project.join("deleted.txt"), "keep in backup").unwrap();
    let workspace = folder::prepare(&project, &root.join("private")).unwrap();
    let git = agentconductor::git::Repository::open(&workspace).unwrap();
    let source = folder::load(&workspace).unwrap().unwrap();
    std::fs::write(workspace.join("changed.txt"), "new").unwrap();
    std::fs::remove_file(workspace.join("deleted.txt")).unwrap();
    let candidate = git
        .snapshot(
            &source.base,
            &["changed.txt".into(), "deleted.txt".into()],
            &root.join("index"),
        )
        .unwrap();
    let run = root.join("run");
    std::fs::create_dir(&run).unwrap();
    source.apply(&git, &candidate, &run).unwrap();
    assert_eq!(std::fs::read(project.join("changed.txt")).unwrap(), b"new");
    assert!(!project.join("deleted.txt").exists());
    assert_eq!(
        std::fs::read(run.join("folder-apply/backup/deleted.txt")).unwrap(),
        b"keep in backup"
    );
    assert!(run.join("folder-apply/COMPLETED").is_file());
    assert!(source.apply(&git, &candidate, &run).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

/// A plain folder has no .git, but its history is kept outside it: a planned
/// `git diff --check` test still runs against that history instead of failing every time.
#[tokio::test]
async fn a_git_test_command_runs_against_the_external_history_of_a_plain_folder() {
    let (root, mut plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    plan.tasks[0].test_command = vec!["git".into(), "diff".into(), "--check".into()];
    let report = workflow::run(plan, &project, &root.join("run"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert!(!project.join(".git").exists());
    std::fs::remove_dir_all(root).unwrap();
}

/// The coordinator replaces a test command that cannot work and records the change.
#[tokio::test]
async fn the_coordinator_replaces_a_test_command_that_cannot_work() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    plan.tasks[0].test_command = vec![
        support::fixture().to_string_lossy().into_owned(),
        "fail".into(),
    ];
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.args = vec!["review-command".into()];
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(&dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(failed.state, "failed");
    let repaired = workflow::repair(&dir, 10, failed).await.unwrap();
    assert_eq!(repaired.state, "waiting_approval", "{:?}", repaired.error);
    assert_eq!(repaired.plan.tasks[0].test_command[1], "test");
    let store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    assert_eq!(
        store.document("test_command_revised").unwrap()["to"][1],
        "test"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
