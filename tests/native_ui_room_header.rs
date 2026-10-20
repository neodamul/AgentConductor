#![cfg(feature = "native-ui-probe")]
//! The room carries its brand and run context, without repeating its native window title.

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, epaint::Shape};

#[test]
fn the_room_body_does_not_repeat_its_native_window_title() {
    let snapshot = serde_json::json!({
        "report": {
            "state": "running", "repo": "/tmp/project", "branch": "main",
            "plan": {"goal": "Rotate tokens"}
        },
        "phase": {
            "phase": "tokens", "slug": "auth", "description": "Rotate tokens",
            "phases": [{"id": "tokens", "name": "Authentication"}]
        }
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
        texts.iter().all(|text| text != "Agent Room"),
        "the room repeats its native window title in the page: {texts:?}"
    );
    assert!(
        texts.iter().any(|text| text == "AgentConductor"),
        "{texts:?}"
    );
    assert!(
        texts.iter().any(|text| text == &version),
        "the room header omits the application version {version}: {texts:?}"
    );
    assert!(texts.iter().any(|text| text == "project"), "{texts:?}");
    assert!(
        texts.iter().any(|text| text.contains("Authentication")),
        "the room header omits the linked phase: {texts:?}"
    );
    assert!(
        texts
            .iter()
            .any(|text| text == "Local · CLI authentication"),
        "{texts:?}"
    );
}
