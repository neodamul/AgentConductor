use agentconductor::native_ui_probe as native;
use eframe::egui::{self, accesskit::Role};
use std::{sync::Arc, time::Duration};

#[test]
fn native_probe_controls_have_accessible_names() {
    let context = egui::Context::default();
    context.set_visuals(egui::Visuals::light());
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| native::Probe::default().show(ui));
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

    let mut inputs = nodes
        .iter()
        .filter(|(_, node)| matches!(node.role(), Role::TextInput | Role::MultilineTextInput))
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    inputs.sort();
    let locale = native::appearance::State::default();
    let mut expected_inputs = [
        locale.text("startup.goal"),
        locale.text("startup.project_folder"),
        locale.text("startup.workspace_name"),
    ];
    expected_inputs.sort();
    let visible_text = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    let missing_hints = [
        "startup.workspace_name_hint",
        "startup.project_folder_hint",
        "startup.goal_hint",
        "startup.no_agents_registered_hint",
    ]
    .into_iter()
    .filter(|id| !visible_text.contains(locale.text(id)))
    .collect::<Vec<_>>();
    let buttons = labels(Role::Button);
    let rendered_step_count = visible_text
        .matches(locale.text("startup.step_1_workspace"))
        .count();
    let coordinator_label = locale.text("startup.coordinator_analysis_of_the_whole_goal");
    let repeated_workspace_heading = output
        .shapes
        .iter()
        .filter(|shape| match &shape.shape {
            egui::Shape::Text(text) => {
                text.galley.job.text == locale.text("startup.workspace_settings_5ebc59")
            }
            _ => false,
        })
        .count();
    let cache_button = buttons.contains(&"Preview and clear local cache");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for label in [
        format!("1. {}", locale.text("startup.workspace_settings_5ebc59")),
        format!("2. {}", locale.text("startup.agent_roles")),
        format!("3. {}", locale.text("startup.review_the_plan")),
        locale.text("startup.choose_your_project_folder").into(),
        locale.text("startup.save_workspace").into(),
        locale.text("startup.open_workspace_menu").into(),
        locale.text("startup.settings").into(),
    ] {
        assert!(buttons.contains(&label.as_str()), "{label}");
    }
    let mut isolation = labels(Role::RadioButton);
    isolation.sort();
    let mut expected_isolation = [
        locale.text("startup.git_worktree"),
        locale.text("startup.shared_folder"),
    ];
    expected_isolation.sort();
    assert_eq!(inputs, expected_inputs);
    assert_eq!(isolation, expected_isolation);
    assert!(missing_hints.is_empty(), "missing {missing_hints:?}");
    assert!(
        visible_text.contains(coordinator_label),
        "Workspace settings must expose the planning coordinator selector"
    );
    assert_eq!(
        repeated_workspace_heading, 2,
        "the Workspace settings card title should not be duplicated"
    );
    assert!(
        visible_text.contains(locale.text("startup.what_should_the_team_work_on")),
        "the existing page-level goal prompt should remain above the step tabs"
    );
    assert!(!cache_button, "cache cleanup belongs inside Settings");
    assert_eq!(
        rendered_step_count, 2,
        "the legacy layout identifies the selected stage above the page and inside its card"
    );
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
    let packet = native::workspace_save_request(
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
    let worker = native::RequestWorker::new(room.clone(), egui::Context::default()).unwrap();
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
    let worker = native::RequestWorker::new(room.clone(), egui::Context::default()).unwrap();

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
        native::picked_folder(existing, &serde_json::json!({"picked_folder": null})),
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
    room.dispatch(&native::workspace_save_request(
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
        native::Probe::render_snapshot(&snapshot, ui);
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

#[path = "native_ui_probe/persistence.rs"]
mod persistence;
