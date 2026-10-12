use agentconductor::{folder, workflow};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_plain_folder_runs_three_tasks_and_only_applies_after_approval() {
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
        b"original human file"
    );
    assert!(!project.join(".git").exists());
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
        !completed.repo.join("one.txt").exists(),
        "completed working copy must be compacted"
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
