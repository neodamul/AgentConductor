#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::goals::{
    self, Event, next_phase_request, select_goal_request,
};
use eframe::egui::{
    self, Event as InputEvent, Modifiers, PointerButton, Pos2, Rect, accesskit::Role,
};
use serde_json::{Value, json};

#[test]
fn active_goal_shows_phase_progress_and_emits_next_phase_action() {
    let snapshot = snapshot();
    let context = egui::Context::default();
    context.enable_accesskit();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    for text in [
        "GOALS & PROGRESS",
        "Rotate refresh tokens",
        "1. Core",
        "Completed",
        "2. Documentation",
        "Next",
        "task 0 · verification 0 · completed 1",
    ] {
        assert!(visible.contains(text), "missing {text:?} in {visible:?}");
    }
    let position = button_position(&output, "Plan next phase");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut events = Vec::new();
    for pressed in [true, false] {
        events.push(InputEvent::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        });
    }
    let (mut output, event) = render(&context, &snapshot, events);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_eq!(event, Some(Event::PlanNextPhase));
    assert_eq!(
        serde_json::from_slice::<Value>(&next_phase_request("rev-42")).unwrap(),
        json!({"kind":"next_phase","revision":"rev-42"})
    );
}

#[test]
fn completed_goal_shows_completion_and_no_next_phase_action() {
    let mut snapshot = snapshot();
    snapshot["slugs"][0]["progress"]["current_phase"] = Value::Null;
    snapshot["slugs"][0]["progress"]["phases"][1]["stage"] = json!("completed");
    let context = egui::Context::default();
    context.enable_accesskit();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    assert!(visible.contains("All 2 phases are complete"), "{visible}");
    assert!(!has_button(&output, "Plan next phase"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn phase_already_in_review_does_not_offer_duplicate_planning() {
    let mut snapshot = snapshot();
    snapshot["proposal"]["slug_phase"] = json!("02-docs");
    let context = egui::Context::default();
    context.enable_accesskit();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    assert!(!has_button(&output, "Plan next phase"));
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn next_phase_expands_its_brief_and_planned_task_details() {
    let mut snapshot = snapshot();
    snapshot["slugs"][0]["plan"] = json!({"phases":[
        {"id":"02-docs","brief":"Document migration and rollout.","specs":[
            {"id":"docs","prompt":"Goal:\n- Write the API migration note.","agent":"OpenCode"}
        ]}
    ]});
    let context = egui::Context::default();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for text in [
        "Document migration and rollout.",
        "docs",
        "Write the API migration note.",
        "OpenCode",
    ] {
        assert!(visible.contains(text), "missing {text:?} in {visible:?}");
    }
}

#[test]
fn reviewed_phase_shows_editing_notice_instead_of_duplicate_planned_tasks() {
    let mut snapshot = snapshot();
    snapshot["proposal"]["slug_phase"] = json!("02-docs");
    snapshot["slugs"][0]["plan"] = json!({"phases":[
        {"id":"02-docs","brief":"Document migration and rollout.","specs":[
            {"id":"docs","prompt":"Write the API migration note.","agent":"OpenCode"}
        ]}
    ]});
    let context = egui::Context::default();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(visible.contains("These tasks are open for review and editing below."));
    assert!(!visible.contains("Write the API migration note."));
}

#[test]
fn goal_analysis_is_available_in_a_collapsible_section() {
    let mut snapshot = snapshot();
    snapshot["slugs"][0]["plan"]["analysis"] =
        json!("Inspect the current authentication flow and identify its risks.");
    let context = egui::Context::default();
    context.enable_accesskit();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let position = button_position(&output, "Coordinator analysis of the whole goal");
    assert!(
        visible.contains("Coordinator analysis of the whole goal"),
        "analysis disclosure is missing: {visible}"
    );
    assert!(!visible.contains("Inspect the current authentication flow"));
    let mut events = Vec::new();
    for pressed in [true, false] {
        events.push(InputEvent::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        });
    }
    let (mut output, _) = render(&context, &snapshot, events);
    let expanded = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(expanded.contains("Inspect the current authentication flow"));
}

#[test]
fn goal_progress_shows_phase_mismatch_warnings() {
    let mut snapshot = snapshot();
    snapshot["slugs"][0]["progress"]["mismatches"] =
        json!(["task docs of phase 02-docs is missing from every stage"]);
    let context = egui::Context::default();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        visible.contains("task docs of phase 02-docs is missing from every stage"),
        "goal progress warning is missing: {visible}"
    );
}

#[test]
fn goal_mismatch_warnings_remain_visible_without_phases() {
    let mut snapshot = snapshot();
    snapshot["slugs"][0]["progress"]["current_phase"] = Value::Null;
    snapshot["slugs"][0]["progress"]["phases"] = json!([]);
    snapshot["slugs"][0]["progress"]["mismatches"] = json!(["plan/plan.json is unreadable"]);
    let context = egui::Context::default();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    let visible = visible_text(&output);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        visible.contains("plan/plan.json is unreadable"),
        "phase-less plan warning is missing: {visible}"
    );
}

#[test]
fn another_goal_is_listed_and_selection_emits_revision_bound_request() {
    let mut snapshot = snapshot();
    snapshot["slugs"].as_array_mut().unwrap().push(json!({
        "slug":"billing_api",
        "goal":"Improve billing API",
        "active":false,
        "progress":{"current_phase":null,"phases":[]}
    }));
    let context = egui::Context::default();
    context.enable_accesskit();
    let (mut output, _) = render(&context, &snapshot, Vec::new());
    assert!(visible_text(&output).contains("Improve billing API"));
    let position = button_position(&output, "Select goal: Improve billing API");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut events = Vec::new();
    for pressed in [true, false] {
        events.push(InputEvent::PointerButton {
            pos: position,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        });
    }
    let (mut output, event) = render(&context, &snapshot, events);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_eq!(event, Some(Event::SelectGoal("billing_api".into())));
    assert_eq!(
        serde_json::from_slice::<Value>(&select_goal_request("rev-42", "billing_api")).unwrap(),
        json!({"kind":"select_goal","revision":"rev-42","slug":"billing_api"})
    );
}

fn snapshot() -> Value {
    json!({
        "revision":"rev-42",
        "job":{"state":"idle"},
        "proposal":{"slug_phase":"01-core"},
        "slugs":[{
            "slug":"refresh_tokens",
            "goal":"Rotate refresh tokens",
            "active":true,
            "progress":{"current_phase":"02-docs","phases":[
                {"id":"01-core","name":"Core","stage":"completed","task":[],"verification":[],"completed":["impl"]},
                {"id":"02-docs","name":"Documentation","stage":"task","task":["docs"],"verification":[],"completed":[]}
            ]}
        }]
    })
}

fn render(
    context: &egui::Context,
    snapshot: &Value,
    events: Vec<InputEvent>,
) -> (egui::FullOutput, Option<Event>) {
    let mut event = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 700.0))),
            events,
            ..Default::default()
        },
        |ui| event = goals::show(snapshot, "en", false, ui),
    );
    (output, event)
}

fn visible_text(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn has_button(output: &egui::FullOutput, label: &str) -> bool {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
}

fn button_position(output: &egui::FullOutput, label: &str) -> Pos2 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .unwrap_or_else(|| panic!("missing button {label:?}"))
}
