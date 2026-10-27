#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::mcp::SourceImport;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect};
use serde_json::{Value, json};

#[test]
fn opening_a_full_mcp_source_does_not_push_the_next_record_down() {
    let snapshot = json!({"config": {
        "goal": "jira:EMM-52599 confluence:53553565 confluence:548443335",
        "mcp": {"company": {}}, "goal_sources": {}, "coordinator": "claude"
    }});
    let preview = json!({"records": [
        {"reference":{"kind":"confluence","id":"53553565"},"title":"First",
            "url":"https://example.invalid/1","body":"First full source body","server":"company","tool":"get_page"},
        {"reference":{"kind":"confluence","id":"548443335"},"title":"Second",
            "url":"https://example.invalid/2","body":"Second full source body","server":"company","tool":"get_page"}
    ], "failures":[], "summaries":{}, "fetched_at":1});
    let mut state = SourceImport::default();
    state.preview_completed(&snapshot, &preview);
    let mut goal = snapshot["config"]["goal"].as_str().unwrap().to_owned();
    let context = egui::Context::default();
    context.enable_accesskit();

    let before = frame(&context, &mut state, &mut goal, &snapshot, Vec::new());
    let second_before = bounds(&before, "confluence:548443335");
    let trigger = bounds(&before, "Full source snapshot: confluence:53553565");
    clear(before);
    let center = Pos2::new(
        ((trigger.x0 + trigger.x1) / 2.0) as f32,
        ((trigger.y0 + trigger.y1) / 2.0) as f32,
    );
    let _ = frame(
        &context,
        &mut state,
        &mut goal,
        &snapshot,
        vec![Event::PointerMoved(center), pointer(center, true)],
    );
    clear(frame(
        &context,
        &mut state,
        &mut goal,
        &snapshot,
        vec![pointer(center, false)],
    ));
    let opened = frame(&context, &mut state, &mut goal, &snapshot, Vec::new());
    let second_after = bounds(&opened, "confluence:548443335");
    let visible_text = opened
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let source_visible = visible_text
        .iter()
        .any(|text| text == "First full source body");
    clear(opened);

    assert_eq!(
        second_after.y0, second_before.y0,
        "source details must overlay records"
    );
    assert!(
        source_visible,
        "full source popup content is missing: {visible_text:?}"
    );
}

#[test]
fn opening_mcp_import_preview_does_not_push_workspace_controls_down() {
    let snapshot = json!({"revision":"rev-1","config": {
        "goal":"confluence:53553565", "mcp":{"company":{}},
        "goal_sources":{}, "coordinator":"claude"
    }});
    let preview = json!({"id":"preview-1", "records":[
        {"reference":{"kind":"confluence","id":"53553565"},"title":"Security guide",
            "url":"https://example.invalid/guide","body":"Full security guide",
            "server":"company","tool":"get_page"}
    ], "failures":[], "summaries":{"confluence:53553565":{"text":"Security summary"}},"fetched_at":1});
    let mut state = SourceImport::default();
    let mut goal = snapshot["config"]["goal"].as_str().unwrap().to_owned();
    let context = egui::Context::default();
    context.enable_accesskit();
    let before = frame(&context, &mut state, &mut goal, &snapshot, Vec::new());
    let controls_before = bounds(&before, "Execution isolation");
    clear(before);

    state.preview_completed(&snapshot, &preview);
    let opened = frame(&context, &mut state, &mut goal, &snapshot, Vec::new());
    let controls_after = bounds(&opened, "Execution isolation");
    let labels = opened
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    clear(opened);

    assert_eq!(controls_after.y0, controls_before.y0);
    assert!(
        labels.iter().any(|label| label == "Apply selected sources"),
        "preview review window should show its apply action: {labels:?}"
    );
}

fn frame(
    context: &egui::Context,
    state: &mut SourceImport,
    goal: &mut String,
    snapshot: &Value,
    events: Vec<Event>,
) -> egui::FullOutput {
    let mut events = Some(events);
    let mut output = None;
    for _ in 0..2 {
        let next = context.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 700.0))),
                events: events.take().unwrap_or_default(),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    state.show(goal, Some(snapshot), "en", true, ui);
                    ui.label("Execution isolation")
                        .accessible_name("Execution isolation");
                });
            },
        );
        if let Some(previous) = output.replace(next) {
            clear(previous);
        }
    }
    output.unwrap()
}

fn bounds(output: &egui::FullOutput, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing accessible UI node {label:?}"))
}

fn pointer(position: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: position,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::default(),
    }
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
