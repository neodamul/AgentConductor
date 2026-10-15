use agentconductor::native_ui_probe::{Probe, RequestWorker};
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

    assert_eq!(labels(Role::TextInput), ["Goal"]);
    assert_eq!(labels(Role::CheckBox), ["Probe accessibility interaction"]);
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
