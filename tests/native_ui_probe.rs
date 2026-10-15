use agentconductor::native_ui_probe::{
    Probe, RequestWorker, picked_folder, workspace_save_request,
};
use eframe::egui::{self, accesskit::Role};
use std::{sync::Arc, time::Duration};

#[test]
fn native_probe_controls_have_accessible_names() {
    let context = egui::Context::default();
    context.set_visuals(egui::Visuals::light());
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| Probe::default().show(ui));
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = |role| {
        nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>()
    };

    let mut inputs = labels(Role::TextInput);
    inputs.sort();
    assert_eq!(inputs, ["Goal", "Repository path", "Workspace name"]);
    let mut buttons = labels(Role::Button);
    buttons.sort();
    assert_eq!(
        buttons,
        [
            "1. Workspace settings",
            "2. Agent roles",
            "3. Review the plan",
            "Browse for project folder",
            "Save workspace"
        ]
    );
    let mut isolation = labels(Role::RadioButton);
    isolation.sort();
    assert_eq!(isolation, ["Git worktree", "Shared folder"]);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn native_workspace_exposes_the_three_required_workflow_steps() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| Probe::default().show(ui));
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"1. Workspace settings"));
    assert!(labels.contains(&"2. Agent roles"));
    assert!(labels.contains(&"3. Review the plan"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn native_prepare_plan_request_is_bound_to_the_current_startup_revision() {
    let packet = agentconductor::native_ui_probe::workspace::prepare_request("rev-current");
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["kind"], "prepare");
    assert_eq!(value["revision"], "rev-current");
}

#[test]
fn native_workspace_save_request_includes_role_agent_and_observed_model_rules() {
    let mut snapshot = serde_json::json!({
        "revision": "rev-current",
        "config": {"roles": {}}
    });
    let roles = std::collections::BTreeMap::from([(
        "testing".to_owned(),
        vec![agentconductor::roles::RoleAgent {
            agent: "codex".into(),
            model: Some("gpt-test".into()),
        }],
    )]);
    snapshot["config"]["roles"] = serde_json::to_value(roles).unwrap();
    let packet = workspace_save_request(
        &snapshot,
        "Workspace",
        "/repo",
        "Goal",
        agentconductor::plan::WorkspaceMode::Shared,
        &[],
        "codex",
    );
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["config"]["roles"]["testing"][0]["agent"], "codex");
    assert_eq!(value["config"]["roles"]["testing"][0]["model"], "gpt-test");
}

#[test]
fn native_workspace_shows_live_coordinator_progress_and_output_activity() {
    let snapshot = serde_json::json!({
        "config": {"coordinator": "claude"},
        "job": {"kind": "plan", "state": "running", "started_ms": 11_000, "output": {"bytes": 83}}
    });
    assert_eq!(
        agentconductor::native_ui_probe::workspace::planning_status(&snapshot, 16_900),
        Some(("claude".into(), 5, 83))
    );
    assert!(
        agentconductor::native_ui_probe::workspace::planning_status(
            &serde_json::json!({"job":{"kind":"plan","state":"completed"}}),
            16_900
        )
        .is_none()
    );
}

#[test]
fn native_worker_returns_workspace_snapshot_through_rust_dispatch() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-ui-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let executable = std::env::current_exe().unwrap();
    let room = Arc::new(agentconductor::ui::Room::startup(&root, &executable).unwrap());
    let worker = RequestWorker::new(room.clone(), egui::Context::default()).unwrap();
    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    let snapshot = worker
        .replies
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();

    assert!(snapshot["config"].is_object());
    assert_eq!(
        snapshot["root"],
        root.canonicalize().unwrap().to_string_lossy().as_ref()
    );
    drop(worker);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_folder_picker_runs_on_worker_and_returns_selected_path() {
    static PICKER_ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = PICKER_ENV.lock().unwrap();
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-ui-picker-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let workspace = root.join("workspace");
    let project = root.join("project");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&project).unwrap();
    unsafe { std::env::set_var("AGENTCONDUCTOR_PICK_FOLDER", &project) };
    let executable = std::env::current_exe().unwrap();
    let room = Arc::new(agentconductor::ui::Room::startup(&workspace, &executable).unwrap());
    let worker = RequestWorker::new(room.clone(), egui::Context::default()).unwrap();

    worker.pick_folder().unwrap();
    let picked = worker
        .replies
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();

    unsafe { std::env::remove_var("AGENTCONDUCTOR_PICK_FOLDER") };
    assert_eq!(picked["picked_folder"], project.to_str().unwrap());
    drop(worker);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cancelled_folder_picker_preserves_existing_repository_path() {
    let existing = "/tmp/current-project";
    assert_eq!(
        picked_folder(existing, &serde_json::json!({"picked_folder": null})),
        existing
    );
}

#[test]
fn native_workspace_summary_renders_the_saved_goal_from_rust_snapshot() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-native-ui-summary-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let executable = std::env::current_exe().unwrap();
    let room = agentconductor::ui::Room::startup(&root, &executable).unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    std::fs::create_dir(root.join("repository")).unwrap();
    room.dispatch(&workspace_save_request(
        &before,
        "Auth workspace",
        root.join("repository").to_str().unwrap(),
        "Rotate refresh tokens",
        agentconductor::plan::WorkspaceMode::Shared,
        &[],
        "",
    ))
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        Probe::render_snapshot(&snapshot, ui);
    });
    let goal_is_visible = output.shapes.iter().any(|shape| match &shape.shape {
        egui::epaint::Shape::Text(text) => text.galley.job.text == "Rotate refresh tokens",
        _ => false,
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(goal_is_visible);
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

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
    let worker = RequestWorker::new(room.clone(), egui::Context::default()).unwrap();
    worker
        .send(&workspace_save_request(
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

    room.dispatch(&workspace_save_request(
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

    room.dispatch(&workspace_save_request(
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
    let mut probe = Probe::connect_room(room.clone(), context.clone()).unwrap();
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
    assert!(labels.iter().any(|label| label.contains("Agent Room")));
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Refresh Agent Room"))
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
