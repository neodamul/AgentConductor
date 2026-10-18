#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn failed_run_banner_shows_status_progress_and_last_error() {
    let snapshot = json!({
        "revision":"rev-9",
        "report":{
            "state":"failed",
            "error":"integration tests failed: refresh token reused",
            "plan":{"goal":"Rotate refresh tokens","tasks":[
                {"id":"implementation"}, {"id":"integration"}, {"id":"documentation"}
            ]},
            "tasks":[
                {"id":"implementation","status":"completed"},
                {"id":"integration","status":"failed","error":"integration tests failed: refresh token reused"},
                {"id":"documentation","status":"pending"}
            ]
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

    assert!(visible.contains("Failed"), "{visible}");
    assert!(visible.contains("Tasks completed: 1/3"), "{visible}");
    assert!(
        visible.contains("integration tests failed: refresh token reused"),
        "{visible}"
    );
}

#[test]
fn fix_with_feedback_selects_failed_task_and_drafts_its_test_output() {
    let snapshot = json!({
        "revision":"rev-9",
        "report":{
            "state":"failed",
            "error":"integration test failed: integration",
            "plan":{"goal":"Rotate refresh tokens","tasks":[
                {"id":"implementation"}, {"id":"integration"}
            ]},
            "tasks":[
                {"id":"implementation","status":"completed"},
                {"id":"integration","status":"failed","error":"assertion failed"}
            ],
            "tests":[{"task":"integration","exit_code":1,"timed_out":false,
                "output":"refresh token was reused"}]
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut first = render(&context, &mut state, &snapshot, Vec::new());
    let button = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .find(|(_, node)| node.label() == Some("Fix with feedback"))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    assert!(button.is_some(), "failed-run action is missing");
    drop(first);

    for pressed in [true, false] {
        let position = button.unwrap();
        let mut output = render(
            &context,
            &mut state,
            &snapshot,
            vec![
                Event::PointerMoved(position),
                Event::PointerButton {
                    pos: position,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    assert_eq!(state.task_controls.selected_index(&snapshot), Some(1));
    let mut output = render(&context, &mut state, &snapshot, Vec::new());
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(visible.contains("refresh token was reused"), "{visible}");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn render(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1100.0, 750.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            room::show(state, Some(snapshot), false, None, "en", ui);
        },
    )
}
