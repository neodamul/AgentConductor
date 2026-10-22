#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::room::{
    judgments,
    task_activity::{self, Tab},
};
use eframe::egui;
use serde_json::json;

#[test]
fn finishing_the_last_watched_task_hands_focus_to_the_coordinator() {
    use agentconductor::native_ui_probe::room;
    let context = egui::Context::default();
    let mut snapshot = json!({"report":{"state":"running","plan":{
        "tasks":[{"id":"one","agent":"company"}],"agents":[]},
        "tasks":[{"id":"one","status":"running"}]}});
    let mut state = room::State::default();
    for (run, task) in [("running", "running"), ("waiting_approval", "completed")] {
        snapshot["report"]["state"] = run.into();
        snapshot["report"]["tasks"][0]["status"] = task.into();
        let mut output = context.run_ui(Default::default(), |ui| {
            room::show(&mut state, Some(&snapshot), false, None, "en", ui);
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    assert!(state.coordinator_selected);
    state.coordinator_selected = false;
    state.task_controls.select_task("one");
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        !state.coordinator_selected,
        "explicitly selected completed work stays selected"
    );
}

#[test]
fn coordinator_selector_follows_the_task_rows() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = json!({"decision_count":2,"report":{"state":"running","plan":{
        "tasks":[{"id":"one","agent":"company"}],"agents":[{"id":"company","parser":"custom"}]},
        "tasks":[{"id":"one","status":"pending"}]}});
    let mut state = agentconductor::native_ui_probe::room::State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::room::show(
            &mut state,
            Some(&snapshot),
            false,
            None,
            "en",
            ui,
        );
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let task_label = agentconductor::ui::messages::interpolate(
        agentconductor::ui::messages::in_language("en", "room.task_status_accessible"),
        &[("value1", "one"), ("value2", "Pending")],
    );
    let task = nodes
        .iter()
        .find(|(_, n)| {
            n.label() == Some(task_label.as_str()) && n.role() == egui::accesskit::Role::Button
        })
        .and_then(|(_, n)| n.bounds())
        .unwrap();
    let coordinator = nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Coordinator judgments: 2"))
        .and_then(|(_, n)| n.bounds())
        .unwrap();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        coordinator.y0 > task.y0,
        "coordinator row must follow tasks: {coordinator:?} vs {task:?}"
    );
}

#[test]
fn a_tasks_activity_renders_its_coordinator_judgment_and_evidence() {
    let context = egui::Context::default();
    let plan = json!({"id":"one.txt"});
    let details = json!({"task":{"id":"one.txt"},"decisions":[{
        "tool":"Bash","input":"{\"command\":\"cargo test\"}","allow":true,"reason":"Run the checks"
    }],"coordinator_decisions":[{
        "kind":"review","judgment":"revise","detail":"Repair the failed test","seq":3
    }]});
    let mut tab = Tab::Activity;
    let mut output = context.run_ui(Default::default(), |ui| {
        task_activity::show(&mut tab, "custom", &plan, Some(&details), None, "en", ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(t) => Some(t.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        text.contains("Coordinator review: revise") && text.contains("Repair the failed test"),
        "{text}"
    );
    assert!(
        text.contains("cargo test"),
        "coordinator judgments must not hide the original tool input: {text}"
    );
}

#[test]
fn judgment_labels_cover_each_kind_in_both_languages() {
    for (kind, verdict, id) in [
        ("tool", "allow", "room.judgment_tool_allow"),
        ("tool", "deny", "room.judgment_tool_deny"),
        ("tool", "undecided", "room.judgment_tool_undecided"),
        ("review", "accept", "room.judgment_review_accept"),
        ("review", "revise", "room.judgment_review_revise"),
        ("review", "blocked", "room.judgment_review_blocked"),
        ("amendment", "applied", "room.judgment_amendment_applied"),
        ("amendment", "undone", "room.judgment_amendment_undone"),
        (
            "amendment",
            "escalated",
            "room.judgment_amendment_escalated",
        ),
        (
            "test_command",
            "replaced",
            "room.judgment_test_command_replaced",
        ),
        ("timeout", "resume", "room.judgment_timeout_resume"),
        ("handoff", "reassigned", "room.judgment_handoff_reassigned"),
        ("lesson", "recorded", "room.judgment_lesson_recorded"),
        ("gate", "passed", "room.judgment_gate_passed"),
        ("goal_drift", "blocked", "room.judgment_goal_drift"),
    ] {
        for language in ["en", "ko"] {
            let expected = agentconductor::ui::messages::in_language(language, id);
            assert_ne!(expected, id);
            assert_eq!(
                judgments::label(&json!({"kind":kind,"judgment":verdict}), language),
                expected
            );
        }
    }
    assert_eq!(
        judgments::label(&json!({"kind":"recovery","judgment":"resumed"}), "en"),
        "Recovery: resumed"
    );
    assert_eq!(
        judgments::label(&json!({"kind":"new","judgment":"future"}), "en"),
        "Coordinator judgment: new future"
    );
}

#[test]
fn only_active_applied_amendments_show_an_undo_control() {
    let context = egui::Context::default();
    let data = json!({"judgments":[
        {"kind":"amendment","judgment":"applied","id":"am-1","undone":false,"task":"auth","detail":"extra.rs"},
        {"kind":"amendment","judgment":"applied","id":"am-2","undone":true,"task":"docs","detail":"notes.md"},
        {"kind":"amendment","judgment":"escalated","id":"am-3","task":"secret","detail":"sensitive path"}
    ]});
    let mut output = context.run_ui(Default::default(), |ui| {
        let _ = judgments::view(&data, "en", ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_eq!(text.matches("Undo amendment").count(), 1, "{text}");
}

#[test]
fn clicking_undo_emits_the_selected_amendment_id() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let data = json!({"judgments":[{
        "kind":"amendment","judgment":"applied","id":"am-undo-this",
        "undone":false,"task":"auth","detail":"rotation.rs"
    }]});
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 700.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(input.clone(), |ui| {
        let _ = judgments::view(&data, "en", ui);
    });
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Undo amendment"))
        .and_then(|(_, node)| node.bounds())
        .expect("undo button is accessible");
    let position = egui::Pos2::new(
        (bounds.x0 + bounds.x1) as f32 / 2.0,
        (bounds.y0 + bounds.y1) as f32 / 2.0,
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut selected = None;
    for pressed in [true, false] {
        let mut output = context.run_ui(
            egui::RawInput {
                events: vec![egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..input.clone()
            },
            |ui| {
                selected = judgments::view(&data, "en", ui);
            },
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    assert_eq!(selected.as_deref(), Some("am-undo-this"));
}

#[test]
fn judgment_view_discloses_its_bound_and_whether_a_change_was_applied() {
    let context = egui::Context::default();
    let data = json!({"decision_count":2001,"judgments":[{
        "kind":"review","judgment":"blocked","detail":"Keep the approved scope",
        "summary":"Escalation required","applied":false
    },{"kind":"goal_drift","judgment":"blocked","detail":"The approved charter changed"}]});
    let mut output = context.run_ui(Default::default(), |ui| {
        let _ = judgments::view(&data, "en", ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(t) => Some(t.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for expected in [
        "Showing the latest 2 of 2001 judgments",
        "Coordinator review: blocked",
        "Keep the approved scope",
        "Escalation required",
        "Not applied",
        "Coordinator blocked the run because its approved charter changed",
        "The approved charter changed",
    ] {
        assert!(text.contains(expected), "{expected}: {text}");
    }
}
