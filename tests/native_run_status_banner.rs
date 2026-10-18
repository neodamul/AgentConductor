#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui;
use serde_json::json;

#[test]
fn failed_run_banner_shows_status_progress_and_last_error() {
    let snapshot = json!({
        "revision":"rev-9",
        "report":{
            "state":"failed",
            "error":"integration tests failed: refresh token reused",
            "plan":{"goal":"Rotate refresh tokens","tasks":[
                {"id":"implementation"}, {"id":"integration"}, {"id":"documentation"}
            ]},
            "tasks":[
                {"id":"implementation","status":"completed"},
                {"id":"integration","status":"failed","error":"integration tests failed: refresh token reused"},
                {"id":"documentation","status":"pending"}
            ]
        }
    });
    let context = egui::Context::default();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    assert!(visible.contains("Failed"), "{visible}");
    assert!(visible.contains("Tasks completed: 1/3"), "{visible}");
    assert!(
        visible.contains("integration tests failed: refresh token reused"),
        "{visible}"
    );
}
