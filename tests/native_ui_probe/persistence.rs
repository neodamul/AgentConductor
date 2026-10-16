use super::*;

#[test]
fn native_workspace_save_uses_the_async_worker_and_refreshes_persisted_state() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-ui-save-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let executable = std::env::current_exe().unwrap();
    let room = Arc::new(agentconductor::ui::Room::startup(&root, &executable).unwrap());
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    std::fs::create_dir(root.join("repository")).unwrap();
    let worker = native::RequestWorker::new(room.clone(), egui::Context::default()).unwrap();
    worker
        .send(&native::workspace_save_request(
            &before,
            "Auth workspace",
            root.join("repository").to_str().unwrap(),
            "Rotate refresh tokens",
            agentconductor::plan::WorkspaceMode::Shared,
            &[],
            "",
        ))
        .unwrap();
    assert_eq!(
        worker
            .replies
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap()["saved"],
        true
    );
    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    let after = worker
        .replies
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();

    assert_eq!(after["config"]["goal"], "Rotate refresh tokens");
    assert_eq!(after["config"]["name"], "Auth workspace");
    assert_eq!(
        after["config"]["repository"],
        root.join("repository")
            .canonicalize()
            .unwrap()
            .to_str()
            .unwrap()
    );
    drop(worker);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_workspace_save_persists_the_selected_isolation_mode() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-ui-mode-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let repository = root.join("repository");
    std::fs::create_dir_all(&repository).unwrap();
    let room = agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();

    room.dispatch(&native::workspace_save_request(
        &before,
        "Isolated workspace",
        repository.to_str().unwrap(),
        "Run isolated tasks",
        agentconductor::plan::WorkspaceMode::Worktree,
        &[],
        "",
    ))
    .unwrap();

    let after = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(after["config"]["workspace_mode"], "worktree");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_profile_templates_get_unique_ids_and_keep_the_cli_invocation() {
    let template = serde_json::json!({
        "id": "claude",
        "executable": "claude",
        "args": ["--print", "{task_file}"],
        "parser": "claude"
    });
    let first = agentconductor::native_ui_probe::profile_from_template(&template, &[]).unwrap();
    let second = agentconductor::native_ui_probe::profile_from_template(
        &template,
        std::slice::from_ref(&first),
    )
    .unwrap();

    assert_eq!(first["id"], "claude");
    assert_eq!(second["id"], "claude-2");
    assert_eq!(second["executable"], template["executable"]);
    assert_eq!(second["args"], template["args"]);
    let full = (0..16)
        .map(|index| serde_json::json!({"id": format!("agent-{index}")}))
        .collect::<Vec<_>>();
    assert!(agentconductor::native_ui_probe::profile_from_template(&template, &full).is_none());
}

#[test]
fn native_team_save_persists_profiles_and_coordinator() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-ui-team-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let repository = root.join("repository");
    std::fs::create_dir_all(&repository).unwrap();
    let room = agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let templates = before["templates"].as_array().unwrap();
    let profiles = ["claude", "codex"]
        .iter()
        .map(|id| templates.iter().find(|p| p["id"] == *id).unwrap().clone())
        .collect::<Vec<_>>();

    room.dispatch(&native::workspace_save_request(
        &before,
        "Agent team",
        repository.to_str().unwrap(),
        "Coordinate three workstreams",
        agentconductor::plan::WorkspaceMode::Shared,
        &profiles,
        "codex",
    ))
    .unwrap();

    let after = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(after["config"]["profiles"].as_array().unwrap().len(), 2);
    assert_eq!(after["config"]["coordinator"], "codex");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_agent_room_probe_opens_the_real_controller_without_workspace_controls() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-room-probe-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let repo = root.join("repo");
    let run = root.join("run");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir(&run).unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
    ] {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    std::fs::write(repo.join("base.txt"), "base\n").unwrap();
    for args in [vec!["add", "."], vec!["commit", "-m", "base"]] {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let report = serde_json::json!({
        "schema_version": 1,
        "state": "waiting_availability",
        "repo": repo,
        "base": "base",
        "branch": "main",
        "plan": {"goal": "Inspect the native room", "agents": [], "tasks": [], "workspace_mode": "shared"},
        "tasks": [],
        "candidate": null,
        "tests": [],
        "diff_hash": null,
        "error": null,
        "verification_cwd": root.join("repo")
    });
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "native Agent Room probe fixture")
        .unwrap();
    let room =
        Arc::new(agentconductor::ui::Room::open(&run, &std::env::current_exe().unwrap()).unwrap());
    let actual = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(actual["report"]["plan"]["goal"], "Inspect the native room");
    let stop =
        agentconductor::native_ui_probe::room::stop_request(&actual, "Operator requested stop")
            .unwrap();
    assert_eq!(room.dispatch(&stop).unwrap()["accepted"], true);
    assert_eq!(
        room.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["report"]["state"],
        "stopped"
    );

    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = native::Probe::connect_surface(
        room.clone(),
        context.clone(),
        agentconductor::native_ui_probe::Surface::AgentRoom,
    )
    .unwrap();
    let mut output = context.run_ui(Default::default(), |ui| probe.show(ui));
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let language = agentconductor::ui::language::load();
    let refresh_label = agentconductor::ui::messages::in_language(&language, "room.refresh_run");
    assert!(
        labels.contains(&refresh_label),
        "{refresh_label}; {labels:?}"
    );
    assert!(!labels.contains(&"Workspace name"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    drop(output);
    drop(probe);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}
