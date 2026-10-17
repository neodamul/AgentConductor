use agentconductor::native_ui_probe::room;
use eframe::egui;
use serde_json::json;

#[test]
fn native_room_preserves_accessible_summary_activity_and_inspector_controls() {
    let snapshot = json!({
        "revision":"r1",
        "job":{"state":"completed"},
        "report":{
            "state":"completed",
            "plan":{
                "goal":"Verify the native Agent Room layout",
                "agents":[{"id":"local","parser":"custom"}],
                "tasks":[{"id":"task-1","agent":"local","prompt":"Inspect the task"}]
            },
            "tasks":[{"id":"task-1","status":"completed","session_id":"fixture-session"}],
            "candidate":null
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1200.0, 800.0),
        )),
        ..Default::default()
    };
    let mut state = room::State::default();
    let mut output = context.run_ui(input, |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let text_positions = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some((text.galley.job.text.as_str(), text.pos.x)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let x = |label: &str| {
        text_positions
            .iter()
            .find_map(|(text, x)| (*text == label).then_some(*x))
            .unwrap_or_else(|| panic!("missing {label:?}"))
    };
    assert!(
        x("Verify the native Agent Room layout") < x("Activity")
            && x("Activity") < x("Task inspector"),
        "summary, activity and inspector must render as three left-to-right columns"
    );
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"Run status: Completed"));
    assert!(labels.contains(&"Activity"));
    assert!(labels.contains(&"Task inspector"));
}
