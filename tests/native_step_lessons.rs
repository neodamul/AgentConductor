#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::task_activity::{self, Tab};
use eframe::egui;
use serde_json::json;

#[test]
fn agent_room_lists_step_lessons_with_cause_and_response() {
    let plan = json!({"id":"integration"});
    let details = json!({
        "task":{"id":"integration"},
        "knowledge":[],
        "step_lessons":[{
            "task":"implementation",
            "cause":"the mock command used the wrong result event",
            "response":"emit a Custom result event"
        }]
    });
    let mut tab = Tab::Lessons;
    let mut output = egui::Context::default().run_ui(Default::default(), |ui| {
        task_activity::show(&mut tab, "codex", &plan, Some(&details), None, "en", ui);
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
    assert!(text.contains("implementation"));
    assert!(text.contains("the mock command used the wrong result event"));
    assert!(text.contains("emit a Custom result event"));
}
