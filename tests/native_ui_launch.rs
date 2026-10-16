use agentconductor::native_ui_probe::{Probe, Surface};
use eframe::egui;
use std::{fs, sync::Arc, time::Duration};

fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "agentconductor-native-surface-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn workspace_launch_surface_connects_to_workspace_controls() {
    let root = temp_dir();
    let room = Arc::new(
        agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
    );
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe =
        Probe::connect_surface(room.clone(), context.clone(), Surface::Workspace).unwrap();
    let mut output = context.run_ui(Default::default(), |ui| probe.show(ui));
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"Workspace name"));
    assert!(!labels.contains(&"Agent Room"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    drop(output);
    drop(probe);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    fs::remove_dir_all(root).unwrap();
}
