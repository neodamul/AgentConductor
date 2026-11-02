use agentconductor::native_ui_probe::room;
use eframe::egui;
use serde_json::json;

#[test]
fn room_session_list_has_a_localized_navigation_heading_and_count() {
    let snapshot = json!({
        "report": {
            "state": "running",
            "plan": {
                "goal": "Review session navigation",
                "agents": [{"id": "local", "parser": "custom"}],
                "tasks": [
                    {"id": "task-1", "agent": "local", "prompt": "First"},
                    {"id": "task-2", "agent": "local", "prompt": "Second"}
                ]
            },
            "tasks": [
                {"id": "task-1", "status": "running", "session_id": "s1"},
                {"id": "task-2", "status": "pending"}
            ]
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200.0, 800.0),
            )),
            ..Default::default()
        },
        |ui| {
            room::show(
                &mut room::State::default(),
                Some(&snapshot),
                false,
                None,
                "en",
                ui,
            );
        },
    );
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = nodes
        .iter()
        .filter_map(|(_, node)| node.label().or_else(|| node.value()))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.iter().any(|label| label == "YOUR SESSIONS"));
    assert!(labels.iter().any(|label| label == "2"));
}
