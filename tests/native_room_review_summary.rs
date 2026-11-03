use agentconductor::native_ui_probe::room::{self, Event, State, task_activity::Tab};
use eframe::egui::{self, Pos2};
use serde_json::json;

#[test]
fn sidebar_review_link_opens_current_candidate_evidence() {
    let snapshot = json!({
        "revision": "rev-9",
        "tool_requests": [{"id": "one"}, {"id": "two"}],
        "report": {
            "state": "waiting_approval",
            "candidate": {"commit": "candidate-9"},
            "plan": {
                "goal": "Review the candidate",
                "tasks": [{"id": "tests", "agent": "codex"}],
                "agents": [{"id": "codex", "parser": "codex"}]
            },
            "tasks": [{"id": "tests", "status": "completed"}],
            "tests": []
        }
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let access = output.platform_output.accesskit_update.as_ref().unwrap();
    let bounds = access
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some("Review candidate"))
        .and_then(|(_, node)| node.bounds());
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(visible.contains("HUMAN REVIEW"), "{visible}");
    assert!(visible.contains("Tool requests"), "{visible}");
    assert!(visible.contains("2"), "{visible}");
    let bounds = bounds.expect("human review summary exposes a candidate button");
    let pos = Pos2::new(
        (bounds.x0 + bounds.x1) as f32 / 2.0,
        (bounds.y0 + bounds.y1) as f32 / 2.0,
    );
    let mut result = None;
    for pressed in [true, false] {
        let mut output = context.run_ui(
            egui::RawInput {
                events: vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            },
            |ui| result = room::show(&mut state, Some(&snapshot), false, None, "en", ui),
        );
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
    assert_eq!(result, Some(Event::Evidence));
    assert_eq!(state.activity_tab, Tab::Review);
}
