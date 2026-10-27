#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::goals;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect, accesskit::Role};
use serde_json::{Value, json};

#[test]
fn opening_phase_details_does_not_push_other_phases_or_actions_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = snapshot();
    let baseline = frame(&context, &snapshot, Vec::new());
    let phase = bounds_containing(&baseline, "2. Documentation");
    let action = bounds(&baseline, "Plan phase 2");
    let details = bounds_containing(&baseline, "1. Core");
    clear(baseline);

    let center = Pos2::new(
        ((details.x0 + details.x1) / 2.0) as f32,
        ((details.y0 + details.y1) / 2.0) as f32,
    );
    let _ = frame(
        &context,
        &snapshot,
        vec![Event::PointerMoved(center), pointer(center, true)],
    );
    clear(frame(&context, &snapshot, vec![pointer(center, false)]));
    let opened = frame(&context, &snapshot, Vec::new());
    let phase_after = bounds_containing(&opened, "2. Documentation");
    let action_after = bounds(&opened, "Plan phase 2");
    let visible = visible_text(&opened);
    let nodes = &opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let accessible = nodes
        .iter()
        .any(|(_, node)| node.label() == Some("Implement core behavior"));
    let control_accessible = nodes.iter().any(|(_, node)| {
        node.role() == Role::Button && node.label().is_some_and(|label| label.contains("1. Core"))
    });
    clear(opened);

    assert_eq!(
        phase_after.y0, phase.y0,
        "phase details must overlay plan rows"
    );
    assert_eq!(
        action_after.y0, action.y0,
        "phase details must overlay the action"
    );
    assert!(visible.iter().any(|text| text == "Implement core behavior"));
    assert!(
        accessible,
        "phase details must remain available to assistive technology"
    );
    assert!(
        control_accessible,
        "phase controls must remain accessible buttons"
    );
}

#[test]
fn opening_next_phase_exposes_brief_and_task_details() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut snapshot = snapshot();
    snapshot["slugs"][0]["plan"] = json!({"phases":[
        {"id":"02-docs","brief":"Document migration and rollout.","specs":[
            {"id":"docs","prompt":"Goal:\n- Write the API migration note.","agent":"OpenCode"}
        ]}
    ]});
    let opened = open(&context, &snapshot, "2. Documentation");
    let visible = visible_text(&opened);
    let accessible = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| {
            node.label()
                .is_some_and(|label| label.contains("Write the API migration note."))
        });
    clear(opened);
    for text in [
        "Document migration and rollout.",
        "docs",
        "Write the API migration note.",
        "OpenCode",
    ] {
        assert!(
            visible.iter().any(|line| line.contains(text)),
            "missing {text:?} in {visible:?}"
        );
    }
    assert!(accessible, "planned task details must remain accessible");
}

#[test]
fn opening_reviewed_phase_shows_editing_notice_without_duplicate_tasks() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut snapshot = snapshot();
    snapshot["proposal"]["slug_phase"] = json!("02-docs");
    snapshot["slugs"][0]["plan"] = json!({"phases":[
        {"id":"02-docs","brief":"Document migration and rollout.","specs":[
            {"id":"docs","prompt":"Write the API migration note.","agent":"OpenCode"}
        ]}
    ]});
    let opened = open(&context, &snapshot, "2. Documentation");
    let visible = visible_text(&opened);
    clear(opened);
    assert!(
        visible
            .iter()
            .any(|line| line.contains("These tasks are open for review and editing below."))
    );
    assert!(
        !visible
            .iter()
            .any(|line| line.contains("Write the API migration note."))
    );
}

fn snapshot() -> Value {
    json!({
        "job":{"state":"idle"}, "proposal":{"slug_phase":null},
        "slugs":[{"slug":"auth", "goal":"Refresh token rotation", "active":true,
            "progress":{"current_phase":"02-docs", "mismatches":[], "phases":[
                {"id":"01-core", "name":"Core", "stage":"completed", "task":[], "verification":[], "completed":["impl"]},
                {"id":"02-docs", "name":"Documentation", "stage":"task", "task":["docs"], "verification":[], "completed":[]}
            ]},
            "plan":{"phases":[
                {"id":"01-core", "brief":"Implement core behavior", "specs":[]},
                {"id":"02-docs", "brief":"Document behavior"}
            ]}
        }]
    })
}

fn frame(context: &egui::Context, snapshot: &Value, events: Vec<Event>) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 700.0))),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                goals::show(snapshot, "en", false, ui);
            });
        },
    )
}

fn bounds(output: &egui::FullOutput, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {label:?}"))
}

fn bounds_containing(output: &egui::FullOutput, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label().is_some_and(|text| text.contains(label)))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing control containing {label:?}"))
}

fn open(context: &egui::Context, snapshot: &Value, label: &str) -> egui::FullOutput {
    let output = frame(context, snapshot, Vec::new());
    let bounds = bounds_containing(&output, label);
    let center = Pos2::new(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    clear(output);
    let _ = frame(context, snapshot, vec![pointer(center, true)]);
    clear(frame(context, snapshot, vec![pointer(center, false)]));
    frame(context, snapshot, Vec::new())
}

fn visible_text(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::default(),
    }
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
