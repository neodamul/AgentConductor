#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{Probe, appearance::State};
use eframe::egui::{self, Pos2, accesskit::Role};

#[test]
fn workspace_frame_reserves_legacy_insets_in_the_raw_viewport() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::default();
    let locale = State::load();
    let mut output = frame(&context, &mut probe);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = frame(&context, &mut probe);
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let bounds = |role, label: &str| {
        nodes
            .iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("missing {label}"))
    };
    let roster = nodes
        .iter()
        .find(|(_, node)| node.label() == Some(locale.text("startup.cli_agents")))
        .and_then(|(_, node)| node.bounds())
        .expect("missing agent roster heading");
    let folder = bounds(Role::TextInput, locale.text("startup.project_folder"));
    let goal = bounds(Role::MultilineTextInput, locale.text("startup.goal"));
    assert!((roster.x0 - 22.0).abs() <= 2.0, "roster inset: {roster:?}");
    assert!((folder.x0 - 314.0).abs() <= 2.0, "folder inset: {folder:?}");
    assert!((goal.x0 - 314.0).abs() <= 2.0, "goal inset: {goal:?}");
    assert!(
        (goal.x1 - 1215.0).abs() <= 2.0,
        "goal right inset: {goal:?}"
    );
    output.textures_delta.free.clear();
}

fn frame(context: &egui::Context, probe: &mut Probe) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1280.0, 892.0),
            )),
            ..Default::default()
        },
        |ui| probe.show(ui),
    )
}
