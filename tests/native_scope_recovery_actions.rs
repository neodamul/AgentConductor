#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, Event as RoomEvent, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};
use serde_json::json;

#[test]
fn preview_and_approved_apply_buttons_emit_exact_recovery_events() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.scope_failure = Some(json!({
        "error":"out-of-scope change: private.md",
        "offending":["private.md"],
        "tasks":[{"id":"docs","paths":["docs"],"test_command":["cargo","test"]}],
        "stray":[{"path":"private.md","stored":"stray-files/private.md","exists_at_original":false}]
    }));
    state.scope_recovery_open = true;
    state.scope_task = "docs".into();
    state.scope_paths = "docs".into();
    state.scope_test_command = "[\"cargo\",\"test\"]".into();
    let snapshot = json!({
        "revision":"rev-9",
        "job":{"state":"completed"},
        "report":{"state":"failed","error":"out-of-scope change: private.md","tasks":[],"tests":[]}
    });
    render(&context, &mut state, &snapshot, Vec::new());
    let output = render(&context, &mut state, &snapshot, Vec::new());
    let preview = center(bounds(&output, "Preview changes"));
    let event = click(&context, &mut state, &snapshot, preview);
    assert!(
        matches!(event, Some(RoomEvent::ScopeRecoveryPreview { task, paths, test })
        if task == "docs" && paths == ["docs"] && test == Some(vec!["cargo".into(), "test".into()]))
    );

    state.scope_preview = Some(json!({"task":"docs","digest":"sha256:reviewed"}));
    render(&context, &mut state, &snapshot, Vec::new());
    let output = render(&context, &mut state, &snapshot, Vec::new());
    let approve = center(bounds(&output, "Approve and apply"));
    let event = click(&context, &mut state, &snapshot, approve);
    assert!(
        matches!(event, Some(RoomEvent::ScopeRecoveryApply { task, paths, test, digest })
        if task == "docs" && paths == ["docs"] && test == Some(vec!["cargo".into(), "test".into()]) && digest == "sha256:reviewed")
    );
}

fn render(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> egui::FullOutput {
    frame(context, state, snapshot, events).0
}

fn frame(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> (egui::FullOutput, Option<RoomEvent>) {
    let mut result = None;
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1100.0, 750.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                result = room::show(state, Some(snapshot), false, None, "en", ui);
            });
        },
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (output, result)
}

fn click(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    position: Pos2,
) -> Option<RoomEvent> {
    let (mut hover, _) = frame(
        context,
        state,
        snapshot,
        vec![Event::PointerMoved(position)],
    );
    hover.textures_delta.set.clear();
    hover.textures_delta.free.clear();
    let mut result = None;
    for pressed in [true, false] {
        let (mut output, event) = frame(
            context,
            state,
            snapshot,
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
        output.platform_output.accesskit_update.take();
        result = event;
    }
    result
}

fn bounds(output: &egui::FullOutput, label: &str) -> egui::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.label() == Some(label))
        .and_then(|node| node.bounds())
        .map(|rect| {
            egui::Rect::from_min_max(
                Pos2::new(rect.x0 as f32, rect.y0 as f32),
                Pos2::new(rect.x1 as f32, rect.y1 as f32),
            )
        })
        .unwrap_or_else(|| panic!("button is not accessible: {label}"))
}

fn center(rect: egui::Rect) -> Pos2 {
    rect.center()
}
