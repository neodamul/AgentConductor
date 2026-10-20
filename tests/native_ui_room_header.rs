#![cfg(feature = "native-ui-probe")]
//! The native window title already identifies Agent Room; its page body starts at the goal.

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, epaint::Shape};

#[test]
fn the_room_body_does_not_repeat_its_native_window_title() {
    let snapshot = serde_json::json!({
        "report": {"state": "running", "plan": {"goal": "Rotate tokens"}}
    });
    let context = egui::Context::default();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let texts: Vec<String> = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect();
    assert!(
        texts.iter().all(|text| text != "Agent Room"),
        "the room repeats its native window title in the page: {texts:?}"
    );
}
