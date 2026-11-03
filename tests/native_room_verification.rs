use agentconductor::native_ui_probe::room;
use eframe::egui;
use serde_json::json;

#[test]
fn sidebar_summarizes_integration_verification_without_exposing_log_paths() {
    let snapshot = json!({
        "report": {
            "state": "failed",
            "plan": {
                "goal": "Rotate refresh tokens",
                "tasks": [
                    {"id": "implementation", "agent": "claude"},
                    {"id": "integration-tests", "agent": "codex"},
                    {"id": "docs", "agent": "opencode"}
                ]
            },
            "tasks": [],
            "tests": [
                {"task": "implementation", "exit_code": 0, "timed_out": false, "stdout": "/private/tmp/private-output"},
                {"task": "integration-tests", "exit_code": 0, "timed_out": false, "stdout": "/private/tmp/private-output"},
                {"task": "docs", "exit_code": 1, "timed_out": false, "stdout": "/private/tmp/private-output"}
            ]
        }
    });
    let context = egui::Context::default();
    let mut state = room::State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let visible = output
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
    assert!(visible.contains("Integration verification"), "{visible}");
    assert!(visible.contains("2 / 3 task tests passed"), "{visible}");
    assert!(
        !visible.contains("/private/tmp/private-output"),
        "{visible}"
    );
}
