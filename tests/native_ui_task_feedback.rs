#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::task_controls::{self, State};
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn running_task_feedback_editor_is_enabled_during_execution() {
    let snapshot = json!({
        "job":{"state":"running"},
        "report":{
            "state":"running",
            "plan":{
                "tasks":[{"id":"active","agent":"local","prompt":"work"}],
                "agents":[{"id":"local","parser":"custom"}]
            },
            "tasks":[{"id":"active","status":"running"}]
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        task_controls::show(&mut state, &snapshot, false, "en", ui);
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let editor = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::MultilineTextInput && node.label() == Some("YOUR DIRECTION")
        })
        .map(|(_, node)| node)
        .expect("feedback editor should be accessible");
    let disabled = editor.is_disabled();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(!disabled, "running task feedback must be editable");
}
