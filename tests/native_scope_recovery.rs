#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, Event as RoomEvent, State};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn only_scope_failures_offer_the_recovery_review() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let regular = snapshot("integration tests failed");
    let mut output = render(&context, &mut state, &regular, Vec::new());
    assert!(
        !labels(&output)
            .iter()
            .any(|label| label == "Review recovery plan")
    );
    clear(&mut output);
    let scope = snapshot("out-of-scope change: private.md");
    let mut output = render(&context, &mut state, &scope, Vec::new());
    assert!(
        labels(&output)
            .iter()
            .any(|label| label == "Review recovery plan")
    );
    clear(&mut output);
}

#[test]
fn review_action_requests_details_and_modal_keeps_refresh_in_place() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let snapshot = snapshot("out-of-scope change: private.md");
    let mut first = render(&context, &mut state, &snapshot, Vec::new());
    clear(&mut first);
    let mut output = render(&context, &mut state, &snapshot, Vec::new());
    let access = output.platform_output.accesskit_update.as_ref().unwrap();
    let before = bounds(access, "Refresh run");
    let button = center(bounds(access, "Review recovery plan").unwrap());
    clear(&mut output);
    let event = click(&context, &mut state, &snapshot, button);
    assert_eq!(event, Some(RoomEvent::ScopeFailure));

    state.scope_failure = Some(json!({
        "error":"out-of-scope change: private.md",
        "offending":["private.md"],
        "tasks":[{"id":"docs","paths":["docs"],"test_command":["cargo","test"]}],
        "stray":[{"path":"private.md","stored":"stray-files/private.md","exists_at_original":false}]
    }));
    state.scope_task = "docs".into();
    state.scope_paths = "docs".into();
    state.scope_test_command = "[\"cargo\",\"test\"]".into();
    state.scope_preview = Some(json!({
        "task":"docs", "paths":["docs"], "previous_paths":["docs"],
        "previous_test_command":["cargo","test"], "test_command":["cargo","test"],
        "restore":[{"path":"private.md","action":"restore"}], "digest":"sha256:test"
    }));
    state.scope_recovery_open = true;
    let mut first = render(&context, &mut state, &snapshot, Vec::new());
    clear(&mut first);
    let mut output = render(&context, &mut state, &snapshot, Vec::new());
    assert!(
        state.scope_recovery_open,
        "review window unexpectedly closed"
    );
    let visible = text_shapes(&output);
    let after = bounds(
        output.platform_output.accesskit_update.as_ref().unwrap(),
        "Refresh run",
    );
    let button_labels = labels(&output);
    clear(&mut output);
    assert!(
        state.scope_recovery_open,
        "review window unexpectedly closed"
    );
    assert!(visible.contains("private.md"), "{visible}");
    assert!(visible.contains("stray-files/private.md"), "{visible}");
    assert!(visible.contains("Resume it separately"), "{visible}");
    let fields = all_labels(&output);
    assert!(
        fields.iter().any(|label| label.contains("Owned paths")),
        "{fields:?}"
    );
    assert!(
        fields.iter().any(|label| label.contains("Test command")),
        "{fields:?}"
    );
    assert!(
        button_labels.iter().any(|label| label == "Preview changes"),
        "{button_labels:?}"
    );
    assert!(
        button_labels
            .iter()
            .any(|label| label == "Approve and apply"),
        "{button_labels:?}"
    );
    assert_eq!(
        before, after,
        "opening the review must not move page controls"
    );
}

fn snapshot(error: &str) -> serde_json::Value {
    json!({
        "revision":"rev-9",
        "job":{"state":"completed"},
        "report":{
            "state":"failed", "error":error,
            "plan":{"goal":"Review generated docs","tasks":[
                {"id":"docs","paths":["docs"],"test_command":["cargo","test"]}
            ]},
            "tasks":[{"id":"docs","status":"failed","error":error}],
            "tests":[]
        }
    })
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
            egui::CentralPanel::default().show(ui, |ui| {
                room::show(state, Some(snapshot), false, None, "en", ui);
            });
        },
    )
}

fn click(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    position: Pos2,
) -> Option<RoomEvent> {
    let mut result = None;
    let mut hover = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1100.0, 750.0),
            )),
            events: vec![Event::PointerMoved(position)],
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                result = room::show(state, Some(snapshot), false, None, "en", ui);
            });
        },
    );
    clear(&mut hover);
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
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    result = room::show(state, Some(snapshot), false, None, "en", ui);
                });
            },
        );
        clear(&mut output);
    }
    result
}

fn labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|(_, node)| node)
        .filter(|node| node.role() == Role::Button)
        .filter_map(|node| node.label().map(str::to_owned))
        .collect()
}

fn all_labels(output: &egui::FullOutput) -> Vec<String> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect()
}

fn text_shapes(output: &egui::FullOutput) -> String {
    fn collect(shape: &egui::epaint::Shape, text: &mut Vec<String>) {
        match shape {
            egui::epaint::Shape::Text(glyphs) => text.push(glyphs.galley.job.text.clone()),
            egui::epaint::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, text);
                }
            }
            _ => {}
        }
    }
    let mut text = Vec::new();
    for shape in &output.shapes {
        collect(&shape.shape, &mut text);
    }
    text.join(" ")
}

fn bounds(update: &egui::accesskit::TreeUpdate, label: &str) -> Option<egui::Rect> {
    update
        .nodes
        .iter()
        .map(|(_, node)| node)
        .find(|node| node.label() == Some(label))?
        .bounds()
        .map(|rect| {
            egui::Rect::from_min_max(
                Pos2::new(rect.x0 as f32, rect.y0 as f32),
                Pos2::new(rect.x1 as f32, rect.y1 as f32),
            )
        })
}

fn center(rect: egui::Rect) -> Pos2 {
    rect.center()
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
