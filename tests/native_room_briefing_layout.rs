use agentconductor::native_ui_probe::room::{self, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn opening_team_briefing_does_not_push_room_controls_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.briefing = Some(json!({
        "state":"running",
        "observed_at_ms":1700000000000_i64,
        "sessions":[{
            "agent":"Fixture Agent",
            "task":"Fixture task",
            "state":"running",
            "work":"Working on the fixture",
            "session_id":"fixture-session",
            "model":"fixture-model",
            "attempt":1
        }],
        "approvals":[]
    }));
    let snapshot = json!({
        "revision":"r1",
        "report":{
            "state":"running",
            "plan":{"goal":"Verify the briefing popup","agents":[],"tasks":[]},
            "tasks":[]
        }
    });
    let menu_label =
        agentconductor::ui::messages::in_language("en", "room.team_briefing_read_only");
    let before = frame(&context, &mut state, &snapshot, Vec::new());
    let menu = bounds(&before, Role::Button, menu_label);
    let stop_before = bounds(
        &before,
        Role::Button,
        agentconductor::ui::messages::in_language("en", "room.stop_run"),
    );
    clear(before);
    let point = Pos2::new(
        ((menu.x0 + menu.x1) / 2.0) as f32,
        ((menu.y0 + menu.y1) / 2.0) as f32,
    );
    clear(frame(
        &context,
        &mut state,
        &snapshot,
        vec![Event::PointerMoved(point)],
    ));
    let pressed = frame(
        &context,
        &mut state,
        &snapshot,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    clear(pressed);
    let released = frame(&context, &mut state, &snapshot, vec![pointer(point, false)]);
    clear(released);
    let opened = frame(&context, &mut state, &snapshot, Vec::new());
    let stop_after = bounds(
        &opened,
        Role::Button,
        agentconductor::ui::messages::in_language("en", "room.stop_run"),
    );
    let text = opened
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => Some(shape.galley.job.text.as_str().to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    clear(opened);

    assert!(
        text.iter().any(|label| label.contains("Fixture Agent")),
        "the briefing popup should paint its session details: {text:?}"
    );
    assert!(
        (stop_after.y0 - stop_before.y0).abs() < 1.0,
        "opening the briefing popup must not shift the room controls: before={stop_before:?}, after={stop_after:?}"
    );
}

fn frame(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1000.0, 720.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        room::show(state, Some(snapshot), false, None, "en", ui);
                    });
            });
        },
    )
}

fn bounds(output: &egui::FullOutput, role: Role, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {role:?} {label:?}"))
}

fn pointer(point: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
