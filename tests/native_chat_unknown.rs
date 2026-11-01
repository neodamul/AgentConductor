#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::{Event as ChatEvent, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn unknown_chat_send_waits_for_history_then_follows_or_restores() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let accepted = "Accepted question";
    assert!(matches!(
        send(&context, &mut state, accepted),
        Some(ChatEvent::Send(_))
    ));
    state.outcome_unknown("send timed out".into());
    assert!(state.awaiting());

    state.load_snapshot(&serde_json::json!({
        "chat_history":[{"role":"operator","text":accepted}],
        "job":{"id":"job-1","kind":"chat","state":"running"}
    }));
    assert!(state.awaiting());
    assert!(state.completed("job-1", "Late answer".into()));
    assert!(!state.awaiting());

    state.reset();
    state.open();
    let rejected = "Unaccepted question";
    assert!(matches!(
        send(&context, &mut state, rejected),
        Some(ChatEvent::Send(_))
    ));
    state.outcome_unknown("history timed out".into());
    state.load_snapshot(&serde_json::json!({"chat_history":[],"job":null}));
    assert!(!state.awaiting());

    let output = draw(&context, &mut state, Vec::new()).0;
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    assert!(nodes.iter().any(|(_, node)| node.value() == Some(rejected)));
    assert!(!nodes.iter().any(|(_, node)| node.label() == Some(rejected)));
    assert!(
        nodes
            .iter()
            .any(|(_, node)| node.value() == Some("history timed out"))
    );
}

fn send(context: &egui::Context, state: &mut State, question: &str) -> Option<ChatEvent> {
    let output = draw(context, state, Vec::new()).0;
    let input = center(&output, Role::MultilineTextInput, None);
    click(context, state, input);
    let _ = draw(context, state, vec![Event::Paste(question.to_owned())]);
    let output = draw(context, state, Vec::new()).0;
    let send = center(&output, Role::Button, Some("Send"));
    click(context, state, send)
}

fn draw(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<ChatEvent>) {
    let mut event = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            event = state.show("en", ui);
        },
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (output, event)
}

fn center(output: &egui::FullOutput, role: Role, label: Option<&str>) -> Pos2 {
    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == role && label.is_none_or(|label| node.label() == Some(label))
        })
        .map(|(_, node)| node)
        .unwrap();
    let bounds = node.bounds().unwrap();
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn click(context: &egui::Context, state: &mut State, point: Pos2) -> Option<ChatEvent> {
    let pointer = |pressed| Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    let _ = draw(
        context,
        state,
        vec![Event::PointerMoved(point), pointer(true)],
    );
    draw(context, state, vec![pointer(false)]).1
}
