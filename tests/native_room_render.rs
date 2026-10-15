use eframe::egui;
use serde_json::json;

#[test]
fn native_room_snapshot_shows_goal_and_agent_task_statuses() {
    let snapshot = json!({
        "state": "running",
        "plan": {
            "goal": "Rotate refresh tokens",
            "tasks": [
                {"id": "implementation", "agent": "claude"},
                {"id": "integration-tests", "agent": "codex"}
            ]
        },
        "tasks": [
            {"id": "implementation", "status": "running"},
            {"id": "integration-tests", "status": "pending"}
        ]
    });
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::render_snapshot(&snapshot, ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let text = text.join(" ");
    assert!(text.contains("Rotate refresh tokens"));
    assert!(text.contains("implementation"));
    assert!(text.contains("claude"));
    assert!(text.contains("running"));
    assert!(text.contains("integration-tests"));
    assert!(text.contains("pending"));
}

#[test]
fn native_agent_status_colors_distinguish_active_waiting_and_failed() {
    use agentconductor::native_ui_probe::status_color;
    use egui::Color32;

    assert_eq!(
        status_color("waiting_approval", 0.0),
        Color32::from_rgb(218, 148, 34)
    );
    assert_eq!(status_color("failed", 0.0), Color32::from_rgb(205, 55, 55));
    assert_ne!(status_color("running", 0.0), status_color("running", 0.25));
}
