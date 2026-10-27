#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::workspace::chat::{Event as ChatEvent, State};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};
#[test]
fn terminal_input_sends_on_enter_and_shift_enter_keeps_the_draft() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let (output, _) = panel(&context, &mut state, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::MultilineTextInput)
        .unwrap()
        .bounds()
        .unwrap();
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = panel(
        &context,
        &mut state,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    let _ = panel(&context, &mut state, vec![pointer(pos, false)]);
    let _ = panel(&context, &mut state, vec![Event::Text("first line".into())]);
    assert_eq!(state.draft(), "first line");
    let (_, action) = panel(&context, &mut state, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(matches!(action, Some(ChatEvent::Send(question)) if question == "first line"));

    let mut state = State::default();
    state.open();
    let (output, _) = panel(&context, &mut state, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::MultilineTextInput)
        .unwrap()
        .bounds()
        .unwrap();
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = panel(
        &context,
        &mut state,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    let _ = panel(&context, &mut state, vec![pointer(pos, false)]);
    let _ = panel(&context, &mut state, vec![Event::Text("line".into())]);
    let (output, action) = panel(
        &context,
        &mut state,
        vec![key(Key::Enter, Modifiers::SHIFT)],
    );
    assert!(action.is_none());
    assert!(
        output
            .platform_output
            .accesskit_update
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::MultilineTextInput
                && node.value() == Some("line\n"))
    );
}

#[test]
fn enter_in_another_control_does_not_send_the_chat_draft() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let (output, _) = panel(&context, &mut state, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::MultilineTextInput)
        .unwrap()
        .bounds()
        .unwrap();
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = panel(
        &context,
        &mut state,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    let _ = panel(&context, &mut state, vec![pointer(pos, false)]);
    let _ = panel(&context, &mut state, vec![Event::Text("keep draft".into())]);
    let (output, _) = panel(&context, &mut state, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::Button && node.label() == Some("Close"))
        .unwrap()
        .bounds()
        .unwrap();
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = panel(
        &context,
        &mut state,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    let _ = panel(&context, &mut state, vec![pointer(pos, false)]);
    let (_, action) = panel(&context, &mut state, vec![key(Key::Enter, Modifiers::NONE)]);
    assert!(action.is_none());
    assert!(!state.awaiting());
    assert_eq!(state.draft(), "keep draft");
}

#[test]
fn up_recalls_the_previous_question_and_new_conversation_resets() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.restore(&serde_json::json!({"chat_transcript":[{"role":"operator","text":"previous"}]}));
    state.open();
    let (output, _) = panel(&context, &mut state, Vec::new());
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::MultilineTextInput)
        .unwrap()
        .bounds()
        .unwrap();
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = panel(
        &context,
        &mut state,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    let _ = panel(&context, &mut state, vec![pointer(pos, false)]);
    let (_, _) = panel(
        &context,
        &mut state,
        vec![key(Key::ArrowUp, Modifiers::NONE)],
    );
    assert_eq!(state.draft(), "previous");

    let output = panel(&context, &mut state, Vec::new()).0;
    let bounds = output
        .platform_output
        .accesskit_update
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::Button && node.label() == Some("New conversation"))
        .unwrap()
        .bounds()
        .unwrap();
    let pos = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    let _ = panel(
        &context,
        &mut state,
        vec![Event::PointerMoved(pos), pointer(pos, true)],
    );
    let (_, action) = panel(&context, &mut state, vec![pointer(pos, false)]);
    assert!(matches!(action, Some(ChatEvent::Reset)));
    assert!(state.transcript().is_empty());
}

fn panel(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<ChatEvent>) {
    let mut action = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| action = state.show("en", ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (output, action)
}

fn key(key: Key, modifiers: Modifiers) -> Event {
    Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers,
    }
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
