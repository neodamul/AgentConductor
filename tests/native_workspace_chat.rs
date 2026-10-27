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
    assert!(labels.iter().any(|label| label == "새 대화"));
}

#[test]
fn large_conversations_fit_the_explicit_chat_request_bound() {
    let packet = chat::request(
        &serde_json::json!({"revision":"rev-1","language":"en"}),
        "latest question",
    )
    .unwrap();
    assert!(packet.len() <= 128 * 1024);
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["kind"], "chat");
    assert_eq!(value["question"], "latest question");
    assert!(value.get("turns").is_none());
}

#[test]
fn persisted_transcript_and_running_job_restore_when_the_panel_reopens() {
    let mut state = State::default();
    state.restore(&serde_json::json!({
        "chat_transcript":[{"role":"operator","text":"Earlier question"}],
        "job":{"kind":"chat","state":"running","id":"chat-1"}
    }));
    assert!(state.awaiting());
    assert_eq!(state.transcript()[0].text, "Earlier question");
    state.open();
}

#[test]
fn switching_workspace_clears_the_previous_transcript_before_restoring() {
    let mut state = State::default();
    state.restore(&serde_json::json!({
        "config":{"repository":"/repo-a"},
        "chat_transcript":[{"role":"operator","text":"Repo A"}]
    }));
    state.restore(&serde_json::json!({
        "config":{"repository":"/repo-b"},
        "chat_transcript":[]
    }));
    assert!(state.transcript().is_empty());
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
