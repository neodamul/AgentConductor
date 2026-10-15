use agentconductor::native_ui_probe::{Probe, RequestWorker, workspace_save_request};
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

    assert_eq!(labels(Role::TextInput), ["Workspace name", "Goal"]);
    assert_eq!(labels(Role::Button), ["Save workspace"]);
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
    room.dispatch(&workspace_save_request(
        &before,
        "Auth workspace",
        "Rotate refresh tokens",
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
    let worker = RequestWorker::new(room.clone(), egui::Context::default()).unwrap();
    worker
        .send(&workspace_save_request(
            &before,
            "Auth workspace",
            "Rotate refresh tokens",
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
    drop(worker);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}
