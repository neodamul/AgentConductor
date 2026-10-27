#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, Event, State};
use eframe::egui::{self, Event as InputEvent, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn selecting_a_session_row_loads_that_tasks_activity_and_inspector() {
    let snapshot = json!({
        "revision":"r1",
        "report":{
            "state":"running",
            "plan":{"goal":"Ship refresh rotation","tasks":[
                {"id":"implementation","agent":"claude","prompt":"Implement"},
                {"id":"integration","agent":"codex","prompt":"Test"}
            ],"agents":[]},
            "tasks":[
                {"id":"implementation","status":"completed"},
                {"id":"integration","status":"running"}
            ],
            "candidate":null
        },
        "quotas":{},"tool_requests":[],"job":null
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let (first, _) = render(&context, &mut state, &snapshot, Vec::new());
    let position = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .find(|(_, node)| node.label() == Some("Task integration status: Running"))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        });
    clear(first);
    let position = position.expect("session rows must be selectable accessible buttons");

    for pressed in [true, false] {
        clear(
            render(
                &context,
                &mut state,
                &snapshot,
                vec![
                    InputEvent::PointerMoved(position),
                    InputEvent::PointerButton {
                        pos: position,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
            )
            .0,
        );
    }

    assert_eq!(state.task_controls.selected_index(&snapshot), Some(1));
    let (output, event) = render(&context, &mut state, &snapshot, Vec::new());
    assert!(matches!(event, Some(Event::TaskDetails(1))));
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(visible.contains("integration"), "{visible}");
    clear(output);
}

fn render(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    events: Vec<InputEvent>,
) -> (egui::FullOutput, Option<Event>) {
    let mut event = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            events,
            ..Default::default()
        },
        |ui| event = room::show(state, Some(snapshot), false, None, "en", ui),
    );
    (output, event)
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
