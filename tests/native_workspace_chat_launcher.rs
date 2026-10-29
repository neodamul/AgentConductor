#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::State;
use eframe::egui::{self, accesskit::Role};

#[test]
fn coordinator_launcher_is_a_compact_accessible_icon_button() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| state.show_launcher("en", ui));
    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Ask coordinator"))
        .map(|(_, node)| node)
        .expect("localized accessible chat name");
    let bounds = node.bounds().unwrap();
    let painted_robot = output
        .shapes
        .iter()
        .any(|shape| matches!(shape.shape, egui::Shape::Circle(_)));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(bounds.width() <= 40.0);
    assert!(
        painted_robot,
        "the launcher paints a robot face instead of text"
    );
    assert!(!state.is_open());
}
