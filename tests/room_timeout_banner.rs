#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui;
use serde_json::json;

#[test]
fn timed_out_worker_error_is_visible_in_the_native_run_banner() {
    let snapshot = json!({
        "report": {
            "state": "failed",
            "plan": {"tasks": [{"id": "implementation"}]},
            "tasks": [{
                "id": "implementation",
                "status": "failed",
                "error": "worker timed out after 300 seconds"
            }]
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

    assert!(visible.contains("Last error:"), "{visible}");
    assert!(
        visible.contains("worker timed out after 300 seconds"),
        "{visible}"
    );
}
