#![cfg(feature = "native-ui-probe")]
//! The application version belongs to the one titlebar; the Agent Room heading does not
//! repeat it (design guide: one titlebar for the workspace screen and the Agent Room).

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, epaint::Shape};

#[test]
fn the_room_heading_does_not_repeat_the_application_version() {
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
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert!(
        texts.iter().all(|text| !text.contains(&version)),
        "the room repeats {version}: {texts:?}"
    );
}
