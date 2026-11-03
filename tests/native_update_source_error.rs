#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::updates::SourceEditor;
use eframe::egui::{self, Pos2};

#[test]
fn errors_reopen_the_editor_and_remain_accessible() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut editor = SourceEditor::default();
    editor.set_error("Update settings could not be loaded".into());
    let mut render = || {
        context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(800.0, 500.0),
                )),
                ..Default::default()
            },
            |ui| {
                editor.show("en", false, ui);
            },
        )
    };
    let mut first = render();
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = render();
    let labels: Vec<_> = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect();
    assert!(labels.contains(&"Update settings could not be loaded"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
