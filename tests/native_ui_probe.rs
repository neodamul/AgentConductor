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
    let nodes = output.platform_output.accesskit_update.unwrap().nodes;
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
    assert_eq!(buttons, ["Browse for project folder", "Save workspace"]);
    let mut isolation = labels(Role::RadioButton);
    isolation.sort();
    assert_eq!(isolation, ["Git worktree", "Shared folder"]);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
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
    ))
    .unwrap();

    let after = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(after["config"]["workspace_mode"], "worktree");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
