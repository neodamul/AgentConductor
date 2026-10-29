#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::chat::{self, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

#[test]
fn coordinator_chat_drawer_starts_closed_and_opens_from_its_workspace_button() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    assert!(!state.is_open());
    let mut output = frame(&context, &mut state, Vec::new());
    let node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.role() == Role::Button && node.label() == Some("Ask coordinator"))
        .unwrap();
    let bounds = node.bounds().unwrap();
    let point = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    clear(&mut output);
    let _ = frame(
        &context,
        &mut state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    let _ = frame(&context, &mut state, vec![pointer(point, false)]);
    assert!(state.is_open());
}

#[test]
fn answers_are_accepted_only_for_the_active_chat_job() {
    let mut state = State::default();
    state.started("job-1".into());
    assert!(state.awaiting());
    assert!(!state.completed("job-old", "stale answer".into()));
    assert!(state.awaiting());
    assert!(state.completed("job-1", "Current answer".into()));
    assert!(!state.awaiting());
    state.open();
    assert!(state.is_open());
}

#[test]
fn rejected_chat_submission_restores_the_question_as_an_unsent_draft() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let question = "How is this workspace configured?";
    let submitted = submit_question(&context, &mut state, question);
    assert!(matches!(submitted, Some(chat::Event::Send(_))));
    state.rejected("Coordinator job was not accepted".into());

    let mut output = render_chat(&context, &mut state, Vec::new()).0;
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    let restored_draft = tree
        .nodes
        .iter()
        .any(|(_, node)| node.role() == Role::MultilineTextInput && node.value() == Some(question));
    let conversation_contains_unaccepted_turn = tree
        .nodes
        .iter()
        .any(|(_, node)| node.label() == Some(question));
    clear(&mut output);
    assert!(
        restored_draft,
        "the unaccepted question must return to the editor"
    );
    assert!(
        !conversation_contains_unaccepted_turn,
        "an unaccepted question must not remain as a conversation turn"
    );
}

#[test]
fn a_late_accepted_chat_job_keeps_the_question_and_eventually_shows_its_answer() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let question = "How is this workspace configured?";
    let submitted = submit_question(&context, &mut state, question);
    assert!(matches!(submitted, Some(chat::Event::Send(_))));

    state.started("job-late".into());
    assert!(state.awaiting(), "accepted delayed jobs remain followed");
    let mut output = render_chat(&context, &mut state, Vec::new()).0;
    let question_visible = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.label() == Some(question) || node.value() == Some(question))
        || output.shapes.iter().any(|shape| {
            matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == question)
        });
    clear(&mut output);
    assert!(
        question_visible,
        "accepted operator turn remains in chat history"
    );

    assert!(state.completed("job-late", "The repository uses Cargo.".into()));
    let mut output = render_chat(&context, &mut state, Vec::new()).0;
    let rendered = |text: &str| {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some(text) || node.value() == Some(text))
            || output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Text(item) if item.galley.job.text == text)
            })
    };
    let question_visible = rendered(question);
    let answer_visible = rendered("The repository uses Cargo.");
    clear(&mut output);
    assert!(question_visible, "accepted question remains in history");
    assert!(answer_visible, "completed answer appears in history");
    assert!(!state.awaiting());
}

#[test]
fn coordinator_panel_controls_have_korean_labels() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show("ko", ui);
        },
    );
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    clear(&mut output);
    assert!(labels.iter().any(|label| label == "닫기"));
    assert!(labels.iter().any(|label| label == "보내기"));
}

#[test]
fn large_conversations_fit_the_explicit_chat_request_bound() {
    let turns = (0..12)
        .map(|index| agentconductor::chat::Turn {
            role: if index == 11 { "operator" } else { "assistant" }.into(),
            text: "x".repeat(8 * 1024),
        })
        .collect::<Vec<_>>();
    let packet = chat::request(
        &serde_json::json!({"revision":"rev-1","language":"en"}),
        &turns,
    )
    .unwrap();
    assert!(packet.len() <= 128 * 1024);
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["kind"], "chat");
    assert_eq!(value["turns"].as_array().unwrap().len(), 12);
}

fn frame(context: &egui::Context, state: &mut State, events: Vec<Event>) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        },
        |ui| state.show_launcher("en", ui),
    )
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn render_chat(
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

fn control_center(output: &egui::FullOutput, role: Role, label: Option<&str>) -> Pos2 {
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
        .expect("chat control is accessible");
    let bounds = node.bounds().unwrap();
    Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

fn click_chat(context: &egui::Context, state: &mut State, point: Pos2) -> Option<chat::Event> {
    let _ = render_chat(
        context,
        state,
        vec![Event::PointerMoved(point), pointer(point, true)],
    );
    render_chat(context, state, vec![pointer(point, false)]).1
}

fn submit_question(
    context: &egui::Context,
    state: &mut State,
    question: &str,
) -> Option<chat::Event> {
    let mut output = render_chat(context, state, Vec::new()).0;
    let input = control_center(&output, Role::MultilineTextInput, None);
    clear(&mut output);
    click_chat(context, state, input);
    let mut output = render_chat(context, state, vec![Event::Paste(question.to_owned())]).0;
    let send = control_center(&output, Role::Button, Some("Send"));
    clear(&mut output);
    click_chat(context, state, send)
}
