#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::updates::State;
use eframe::egui;

#[path = "support/native_ui_updates.rs"]
#[allow(dead_code)]
mod ui_interaction;

#[test]
fn update_wait_panel_shows_elapsed_time_while_checking() {
    let mut state = State::default();
    state.open = true;
    state.apply_status(serde_json::json!({"phase":"checking"}));
    let context = egui::Context::default();
    context.enable_accesskit();
    let output = ui_interaction::render_update(&context, &mut state, "en", false, Vec::new());
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let painted = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        labels.iter().any(|label| label.starts_with("Elapsed "))
            || painted.iter().any(|text| text.starts_with("Elapsed ")),
        "labels={labels:?}, painted={painted:?}"
    );
}
