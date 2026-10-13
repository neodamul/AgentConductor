use agentconductor::{folder, git::Repository};
#[path = "support/workflow.rs"]
mod support;

#[test]
fn reviewed_folder_history_activates_only_the_selected_project() {
    let (root, _) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    std::fs::write(project.join(".env"), "private").unwrap();
    let goal = project.join("goal");
    std::fs::create_dir(&goal).unwrap();
    std::fs::write(goal.join(agentconductor::slugs::MARKER), "goal").unwrap();
    std::fs::write(goal.join("plan.md"), "plan").unwrap();
    let planned = folder::repository(&project, &root.join("history")).unwrap();
    let base = planned.head().unwrap();
    assert!(!project.join(".git").exists());
    let actual = folder::activate(&planned, &project).unwrap();
    assert_eq!(actual.root, project.canonicalize().unwrap());
    assert_eq!(actual.head().unwrap(), base);
    assert!(!project.join(".git").exists());
    assert!(actual.is_external());
    assert_eq!(std::fs::read(project.join(".env")).unwrap(), b"private");
    assert_eq!(std::fs::read(goal.join("plan.md")).unwrap(), b"plan");
    std::fs::create_dir(project.join("node_modules")).unwrap();
    std::fs::write(project.join("node_modules/generated"), "dependency").unwrap();
    std::fs::write(project.join(".env.local"), "private").unwrap();
    actual.clean().unwrap();
    assert!(folder::load(&actual.root).unwrap().is_none());
    std::fs::write(actual.root.join("result.txt"), "result").unwrap();
    assert!(project.join("result.txt").exists());
    assert!(!planned.root.join("result.txt").exists());
    assert!(
        actual
            .changed(&base)
            .unwrap()
            .contains(&"result.txt".into())
    );
    drop(actual);
    std::fs::remove_dir_all(folder::history_directory(&project).unwrap()).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn changed_or_rebound_project_never_gets_git_metadata() {
    let (root, _) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    let planned = folder::repository(&project, &root.join("history")).unwrap();
    let other = root.join("other");
    std::fs::create_dir(&other).unwrap();
    assert!(folder::activate(&planned, &other).is_err());
    assert!(!other.join(".git").exists());
    std::fs::write(project.join("base.txt"), "human edit").unwrap();
    assert!(folder::activate(&planned, &project).is_err());
    assert!(!project.join(".git").exists());
    assert_eq!(
        std::fs::read(project.join("base.txt")).unwrap(),
        b"human edit"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn existing_history_is_preserved_and_git_projects_stay_bound() {
    let (root, _) = support::setup("shared");
    let project = root.join("repo");
    let original = Repository::open(&project).unwrap();
    assert_eq!(
        folder::activate(&original, &project)
            .unwrap()
            .head()
            .unwrap(),
        original.head().unwrap()
    );
    let other = root.join("other");
    std::fs::create_dir(&other).unwrap();
    assert!(folder::activate(&original, &other).is_err());
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    let planned = folder::repository(&project, &root.join("history")).unwrap();
    std::fs::create_dir(project.join(".git")).unwrap();
    std::fs::write(project.join(".git/keep"), "human metadata").unwrap();
    assert!(folder::activate(&planned, &project).is_err());
    assert_eq!(
        std::fs::read(project.join(".git/keep")).unwrap(),
        b"human metadata"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn new_baseline_and_rejection_restore_owned_files_without_project_git() {
    use agentconductor::{state::Store, workflow::Report};
    use serde_json::json;
    let (root, plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    std::fs::write(project.join("one.txt"), "old").unwrap();
    let planned = folder::repository(&project, &root.join("planning")).unwrap();
    let git = folder::activate(&planned, &project).unwrap();
    let first = git.head().unwrap();
    std::fs::write(project.join("one.txt"), "human baseline").unwrap();
    let git = folder::repository(&project, &root.join("planning")).unwrap();
    let base = git.head().unwrap();
    assert_ne!(base, first);
    assert_eq!(
        git.command(&["show", &format!("{base}:one.txt")]).unwrap(),
        b"human baseline"
    );
    let run = root.join("run");
    std::fs::create_dir(&run).unwrap();
    std::fs::write(project.join("one.txt"), "candidate").unwrap();
    std::fs::write(project.join("two.txt"), "new").unwrap();
    let scopes = vec!["one.txt".into(), "two.txt".into(), "three.txt".into()];
    let candidate = git
        .snapshot(&base, &scopes, &run.join("candidate.index"))
        .unwrap();
    let report:Report=serde_json::from_value(json!({"schema_version":1,"state":"waiting_approval","repo":git.root,"base":base,"branch":git.branch().unwrap(),"plan":plan,"tasks":[],"candidate":candidate,"tests":[],"diff_hash":null,"error":null,"verification_cwd":git.root})).unwrap();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    store
        .record(
            "test_candidate",
            &json!({"commit":report.candidate.as_ref().unwrap().commit}),
            "candidate fixture",
        )
        .unwrap();
    std::fs::write(project.join("one.txt"), "later manual edit").unwrap();
    assert!(folder::restore_rejected(&git, &report, &run).is_err());
    assert_eq!(
        std::fs::read(project.join("one.txt")).unwrap(),
        b"later manual edit"
    );
    std::fs::write(project.join("one.txt"), "candidate").unwrap();
    folder::restore_rejected(&git, &report, &run).unwrap();
    assert_eq!(
        std::fs::read(project.join("one.txt")).unwrap(),
        b"human baseline"
    );
    assert!(!project.join("two.txt").exists());
    assert!(!project.join(".git").exists());
    assert!(
        run.join("workspace-rejection/folder-apply/COMPLETED")
            .exists()
    );
    git.clean().unwrap();
    drop(store);
    std::fs::remove_dir_all(folder::history_directory(&project).unwrap()).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unborn_git_execution_keeps_the_existing_index_and_legacy_binding() {
    let (root, _) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    assert!(
        std::process::Command::new("git")
            .arg("-C")
            .arg(&project)
            .args(["init", "-b", "main"])
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .arg("-C")
            .arg(&project)
            .args(["add", "base.txt"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let index = std::fs::read(project.join(".git/index")).unwrap();
    let planned = folder::repository(&project, &root.join("planning")).unwrap();
    let actual = folder::activate(&planned, &project).unwrap();
    assert_eq!(actual.root, planned.root);
    assert_eq!(std::fs::read(project.join(".git/index")).unwrap(), index);
    assert!(Repository::open(&project).unwrap().head().is_err());
    assert!(folder::load(&actual.root).unwrap().is_some());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn owned_directory_warns_for_new_deleted_and_changed_files() {
    use agentconductor::{
        state::Store,
        workflow::{Report, TaskReport},
    };
    use serde_json::json;
    let (root, mut plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    std::fs::create_dir(project.join("src")).unwrap();
    std::fs::write(project.join("src/kept.txt"), "worker").unwrap();
    let planned = folder::repository(&project, &root.join("planning")).unwrap();
    let git = folder::activate(&planned, &project).unwrap();
    plan.tasks[0].paths = vec!["src".into()];
    let base = git.head().unwrap();
    let task:TaskReport=serde_json::from_value(json!({"id":plan.tasks[0].id,"status":"completed","cwd":git.root,"base":base,"artifacts":root.join("task"),"started_ms":null,"finished_ms":null,"session_id":null,"error":null,"snapshot":null})).unwrap();
    let report:Report=serde_json::from_value(json!({"schema_version":1,"state":"running","repo":git.root,"base":base,"branch":git.branch().unwrap(),"plan":plan,"tasks":[task],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":git.root})).unwrap();
    let mut store = Store::open(root.join("state.sqlite")).unwrap();
    folder::record_owned_checkpoint(&mut store, &git, &report.tasks[0], &["src".into()]).unwrap();
    assert!(folder::owned_changes(&store, &report).unwrap().is_empty());
    std::fs::create_dir(project.join("src/new-dir")).unwrap();
    std::fs::write(project.join("src/new-dir/added.txt"), "manual").unwrap();
    assert!(
        folder::owned_changes(&store, &report)
            .unwrap()
            .contains(&"src/new-dir/added.txt".into())
    );
    std::fs::remove_file(project.join("src/kept.txt")).unwrap();
    assert!(
        folder::owned_changes(&store, &report)
            .unwrap()
            .contains(&"src/kept.txt".into())
    );
    drop(store);
    std::fs::remove_dir_all(folder::history_directory(&project).unwrap()).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn direct_codex_arguments_keep_permissions_model_and_resume_session() {
    use agentconductor::{adapters, protocol::Provider};
    let (root, mut plan) = support::setup("shared");
    let profile = &mut plan.agents[0];
    profile.parser = Provider::Codex;
    profile.executable = "codex".into();
    profile.args = adapters::full_access_args(Provider::Codex).unwrap();
    profile.permission_mode = "full_access".into();
    profile.catalog = Some(agentconductor::plan::ModelCatalog {
        models: vec!["chosen-model".into()],
        source: "observed fixture".into(),
        queried_at: 1,
        cli_version: "fixture".into(),
    });
    for session in [None, Some("owned-session")] {
        let original =
            adapters::session_args(profile, Some("chosen-model"), session, "prompt.txt").unwrap();
        let changed = adapters::direct_folder_args(Provider::Codex, original.clone()).unwrap();
        assert_eq!(
            changed
                .iter()
                .filter(|a| a.as_str() == "--skip-git-repo-check")
                .count(),
            1
        );
        assert_eq!(
            changed
                .iter()
                .filter(|a| a.as_str() != "--skip-git-repo-check")
                .cloned()
                .collect::<Vec<_>>(),
            original
        );
        assert!(changed.contains(&"--dangerously-bypass-approvals-and-sandbox".into()));
        if session.is_some() {
            assert!(
                changed
                    .iter()
                    .position(|a| a == "--skip-git-repo-check")
                    .unwrap()
                    < changed.iter().position(|a| a == "resume").unwrap()
            );
        }
    }
    let readonly = agentconductor::coordinator::read_only_args(profile, "prompt.txt").unwrap();
    let readonly = adapters::direct_folder_args(Provider::Codex, readonly).unwrap();
    assert!(readonly.contains(&"read-only".into()));
    assert!(!readonly.contains(&"--dangerously-bypass-approvals-and-sandbox".into()));
    assert_eq!(
        adapters::direct_folder_args(Provider::Claude, vec!["-p".into()]).unwrap(),
        vec!["-p"]
    );
    std::fs::remove_dir_all(root).unwrap();
}
