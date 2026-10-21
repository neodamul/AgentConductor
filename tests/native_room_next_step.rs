#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, CandidateDecision, Event as RoomEvent, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn approval_banner_moves_from_review_to_approve_then_merge() {
    let mut snapshot = json!({
        "revision":"rev-9",
        "report":{"state":"waiting_approval","candidate":{"commit":"abc"},
            "plan":{"goal":"Rotate tokens","tasks":[{"id":"implement","agent":"claude"}],
                "agents":[{"id":"claude","catalog":{"models":[]}}]},
            "tasks":[{"id":"implement","status":"completed"}]}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();

    let button = next_button(&context, &mut state, &snapshot, "Review changes");
    let event = click(&context, &mut state, &snapshot, button);
    assert_eq!(event, Some(RoomEvent::Evidence));
    assert_eq!(state.activity_tab, room::task_activity::Tab::Review);

    state.evidence = Some(json!({"revision":"rev-8","diff":"stale diff"}));
    let button = next_button(&context, &mut state, &snapshot, "Review changes");
    assert_eq!(
        click(&context, &mut state, &snapshot, button),
        Some(RoomEvent::Evidence)
    );

    state.evidence = Some(json!({"revision":"rev-9","diff":"verified diff","tests":[]}));
    let button = next_button(&context, &mut state, &snapshot, "Approve candidate");
    click(&context, &mut state, &snapshot, button);
    assert_eq!(state.confirming_decision, Some(CandidateDecision::Approve));
    assert_visible(
        &context,
        &mut state,
        &snapshot,
        "Confirm approval of this verified diff and tests?",
    );

    snapshot["report"]["state"] = json!("approved");
    let button = next_button(&context, &mut state, &snapshot, "Merge candidate");
    click(&context, &mut state, &snapshot, button);
    assert_eq!(state.confirming_decision, Some(CandidateDecision::Merge));
    assert_visible(
        &context,
        &mut state,
        &snapshot,
        "Confirm integrating the exact approved candidate?",
    );
}

#[test]
fn completed_run_banner_offers_return_to_workspace() {
    let snapshot = json!({
        "revision":"rev-10",
        "report":{"state":"completed","plan":{"goal":"Rotate tokens","tasks":[]},"tasks":[]}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.workspace_available = true;
    let button = next_button(&context, &mut state, &snapshot, "Return to Workspace");
    assert_eq!(
        click(&context, &mut state, &snapshot, button),
        Some(RoomEvent::ReturnToWorkspace)
    );
}

#[test]
fn running_run_also_offers_return_to_workspace_without_stopping_it() {
    let snapshot = json!({
        "revision":"rev-live",
        "job":{"kind":"start","state":"running"},
        "report":{"state":"running","plan":{"goal":"Rotate tokens","tasks":[]},"tasks":[]}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.workspace_available = true;
    let button = next_button(&context, &mut state, &snapshot, "Return to Workspace");
    assert_eq!(
        click(&context, &mut state, &snapshot, button),
        Some(RoomEvent::ReturnToWorkspace)
    );
}

#[test]
fn running_run_disables_return_while_a_request_is_pending() {
    let snapshot = json!({
        "job":{"kind":"start","state":"running"},
        "report":{"state":"running","plan":{"goal":"Rotate tokens","tasks":[]},"tasks":[]}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.workspace_available = true;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1100.0, 750.0),
            )),
            ..Default::default()
        },
        |ui| {
            room::show(&mut state, Some(&snapshot), true, None, "en", ui);
        },
    );
    let buttons = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| {
            node.role() == Role::Button && node.label() == Some("Return to Workspace")
        })
        .map(|(_, node)| node.is_disabled())
        .collect::<Vec<_>>();
    assert!(!buttons.is_empty(), "return action remains visible");
    assert!(buttons.iter().all(|disabled| *disabled), "{buttons:?}");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn standalone_completed_room_explains_workspace_window_without_broken_action() {
    let snapshot = json!({"report":{"state":"completed","plan":{"tasks":[]},"tasks":[]}});
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
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
    assert!(visible.contains("Open the Workspace window"), "{visible}");
    assert!(
        !output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Button
                && node.label() == Some("Return to Workspace"))
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn assert_visible(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    expected: &str,
) {
    let mut output = render(context, state, snapshot, Vec::new());
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(visible.contains(expected), "{visible}");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn next_button(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    label: &str,
) -> Pos2 {
    let mut output = render(context, state, snapshot, Vec::new());
    let point = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    point.unwrap_or_else(|| panic!("missing next-step button: {label}"))
}

fn click(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    position: Pos2,
) -> Option<RoomEvent> {
    let mut event = None;
    for pressed in [true, false] {
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    egui::vec2(1100.0, 750.0),
                )),
                events: vec![
                    Event::PointerMoved(position),
                    Event::PointerButton {
                        pos: position,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| event = room::show(state, Some(snapshot), false, None, "en", ui),
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    event
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
