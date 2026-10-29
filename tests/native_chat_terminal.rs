#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::{self, State};
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn transcript_uses_terminal_prompt_markers() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    state.load_snapshot(&json!({"chat_history":[
        {"role":"operator","text":"Question"},
        {"role":"assistant","text":"Answer"}
    ]}));

    let output = frame(&context, &mut state, Vec::new()).0;
    let text = rendered_text(&output);

    assert!(text.contains("> Question"), "{text}");
    assert!(text.contains("< Answer"), "{text}");
    assert!(text.contains(">"), "the input has a prompt marker: {text}");
}

#[test]
fn enter_sends_and_shift_enter_keeps_a_newline() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    focus_editor(&context, &mut state);
    let after_first = frame(&context, &mut state, vec![Event::Text("first".into())]).0;
    assert_eq!(editor_value(&after_first), "first");
    frame(
        &context,
        &mut state,
        vec![key(Key::Enter, Modifiers::SHIFT)],
    );
    let output = frame(&context, &mut state, vec![Event::Text("second".into())]).0;
    assert_eq!(editor_value(&output), "first\nsecond");

    let (_, event) = frame(&context, &mut state, vec![key(Key::Enter, Modifiers::NONE)]);

    assert!(matches!(event, Some(chat::Event::Send(turns))
        if turns.last().is_some_and(|turn| turn.text == "first\nsecond")));
}

#[test]
fn up_and_down_recall_questions_and_restore_the_unsent_draft() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    state.load_snapshot(&json!({"chat_history":[
        {"role":"operator","text":"first question"},
        {"role":"assistant","text":"first answer"},
        {"role":"operator","text":"second question"},
        {"role":"assistant","text":"second answer"}
    ]}));
    focus_editor(&context, &mut state);
    frame(
        &context,
        &mut state,
        vec![Event::Text("unsent draft".into())],
    );

    let latest = value_after_key(&context, &mut state, Key::ArrowUp);
    let earlier = value_after_key(&context, &mut state, Key::ArrowUp);
    let next = value_after_key(&context, &mut state, Key::ArrowDown);
    let restored = value_after_key(&context, &mut state, Key::ArrowDown);

    assert_eq!(latest, "second question");
    assert_eq!(earlier, "first question");
    assert_eq!(next, "second question");
    assert_eq!(restored, "unsent draft");
}

fn value_after_key(context: &egui::Context, state: &mut State, key_value: Key) -> String {
    let output = frame(context, state, vec![key(key_value, Modifiers::NONE)]).0;
    editor_value(&output)
}

fn editor_value(output: &egui::FullOutput) -> String {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::MultilineTextInput)
        .and_then(|(_, node)| node.value())
        .unwrap()
        .to_owned()
}

fn focus_editor(context: &egui::Context, state: &mut State) {
    let mut output = frame(context, state, Vec::new()).0;
    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::MultilineTextInput)
        .map(|(_, node)| node)
        .unwrap();
    let bounds = node.bounds().unwrap();
    let point = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    clear(&mut output);
    frame(
        context,
        state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    frame(context, state, vec![pointer(point, false)]);
}

fn rendered_text(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
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

fn frame(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<chat::Event>) {
    let mut action = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 700.0),
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

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
