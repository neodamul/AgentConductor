#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::{self, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn restored_history_can_start_a_new_conversation_and_disable_chat_until_saved() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.load_snapshot(&serde_json::json!({
        "chat_history":[
            {"role":"operator", "text":"Previous question"},
            {"role":"assistant", "text":"Previous answer"}
        ],
        "job":null
    }));
    state.open();
    let mut output = frame(&context, &mut state, Vec::new()).0;
    assert!(contains_text(&output, "> Previous question"));
    let button = control_center(&output, "New chat");
    clear(&mut output);

    let event = click(&context, &mut state, button);
    assert!(matches!(event, Some(chat::Event::NewConversation)));
    state.reset_pending();
    assert!(
        state.awaiting(),
        "do not accept another turn before reset persists"
    );
    state.reset();
    assert!(!state.awaiting());
    let output = frame(&context, &mut state, Vec::new()).0;
    assert!(!contains_text(&output, "> Previous question"));
}

#[test]
fn reopening_a_running_chat_restores_the_transcript_and_follows_its_job() {
    let mut state = State::default();
    state.load_snapshot(&serde_json::json!({
        "chat_history":[{"role":"operator", "text":"Still running?"}],
        "job":{"kind":"chat", "state":"running", "id":"job-resume"}
    }));
    assert!(state.awaiting());
    assert!(state.completed("job-resume", "Yes, it completed.".into()));
    assert!(!state.awaiting());
}

fn frame(
    context: &egui::Context,
    state: &mut State,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<chat::Event>) {
    let mut event = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| event = state.show("en", ui),
    );
    (output, event)
}

fn contains_text(output: &egui::FullOutput, expected: &str) -> bool {
    output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == expected),
    )
}

fn control_center(output: &egui::FullOutput, label: &str) -> Pos2 {
    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .map(|(_, node)| node)
        .unwrap();
    let bounds = node.bounds().unwrap();
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn click(context: &egui::Context, state: &mut State, point: Pos2) -> Option<chat::Event> {
    let button = |pressed| Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    let _ = frame(
        context,
        state,
        vec![Event::PointerMoved(point), button(true)],
    );
    frame(context, state, vec![button(false)]).1
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
