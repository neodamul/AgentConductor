#![cfg(feature = "native-ui-probe")]

use eframe::egui;
use serde_json::json;

#[test]
fn native_run_snapshot_renders_the_blocked_task_cause() {
    let snapshot = json!({
        "report": {
            "state": "blocked",
            "error": "Task timed out after 1800s; retained evidence is available.",
            "plan": {"goal": "Implement refresh rotation", "tasks": []},
            "tasks": []
        }
    });
    let mut output = egui::Context::default().run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::render_snapshot(&snapshot, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(text.contains("Task timed out after 1800s"));
}
