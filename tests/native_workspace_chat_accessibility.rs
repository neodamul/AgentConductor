#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::State;
use eframe::egui::{self, accesskit::Role};

#[test]
fn coordinator_chat_editor_exposes_its_prompt_as_an_accessible_name() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show("en", ui);
        },
    );
    let accessible_name = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::MultilineTextInput)
        .filter_map(|(_, node)| node.label())
        .any(|label| label.starts_with("Ask about this workspace"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(accessible_name, "chat input needs a stable accessible name");
}
