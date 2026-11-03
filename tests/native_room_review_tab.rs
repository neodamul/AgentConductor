#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::{self, Event, State, task_activity::Tab};
use eframe::egui::{self, Pos2};
use serde_json::json;

fn snapshot(revision: &str) -> serde_json::Value {
    json!({
        "revision":revision,
        "report":{
            "state":"waiting_approval",
            "candidate":{"commit":"candidate-1"},
            "plan":{
                "goal":"Review the token rotation",
                "tasks":[{"id":"tests","agent":"codex","prompt":"Run tests"}],
                "agents":[{"id":"codex","parser":"codex"}]
            },
            "tasks":[{"id":"tests","status":"completed"}]
        }
    })
}

fn labels(output: &egui::FullOutput) -> Vec<String> {
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

fn visible_text(output: &egui::FullOutput) -> String {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn click(
    context: &egui::Context,
    state: &mut State,
    snapshot: &serde_json::Value,
    label: &str,
) -> Option<Event> {
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(state, Some(snapshot), false, None, "en", ui);
    });
    let bounds = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing accessible button {label}"));
    let pos = Pos2::new(
        (bounds.x0 + bounds.x1) as f32 / 2.0,
        (bounds.y0 + bounds.y1) as f32 / 2.0,
    );
    clear(&mut output);
    let mut event = None;
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
            |ui| {
                event = room::show(state, Some(snapshot), false, None, "en", ui);
            },
        );
        clear(&mut output);
    }
    event
}

#[test]
fn review_tab_separates_approval_evidence_and_refreshes_stale_results() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = snapshot("revision-2");
    let mut state = State::default();
    state.details_task = Some("tests".into());
    state.task_details = Some(json!({"task":{"id":"tests"},"events":[]}));
    state.diff = "+rotate refresh tokens\n".into();
    state.evidence = Some(json!({
        "revision":"revision-2",
        "report":{"candidate":{"commit":"candidate-1"},"tests":[{"task":"tests","exit_code":0,"timed_out":false}]},
        "test_output":[{"task":"tests","stdout":"2 tests passed","stderr":""}],
        "diff":"+rotate refresh tokens\n"
    }));
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let initial = labels(&output);
    let initial_text = visible_text(&output);
    assert!(initial.contains(&"Diff & tests".into()), "{initial:?}");
    assert!(initial.contains(&"Activity".into()));
    assert!(!initial_text.contains("2 tests passed"), "{initial_text}");
    clear(&mut output);

    let _ = click(&context, &mut state, &snapshot, "Diff & tests");
    assert_eq!(state.activity_tab, Tab::Review);
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let review = labels(&output);
    assert_eq!(state.activity_tab, Tab::Review);
    let visible = visible_text(&output);
    assert!(
        visible.contains("+rotate refresh tokens"),
        "labels={review:?}; visible={visible:?}"
    );
    assert!(visible.contains("2 tests passed"), "{visible}");
    clear(&mut output);

    state.evidence.as_mut().unwrap()["revision"] = "revision-1".into();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let stale = labels(&output);
    let stale_text = visible_text(&output);
    assert!(
        stale
            .iter()
            .any(|label| label.contains("Run state changed")),
        "{stale:?}"
    );
    assert!(stale.contains(&"Review latest".into()));
    assert!(
        !stale_text.contains("+rotate refresh tokens"),
        "{stale_text}"
    );
    clear(&mut output);
    let event = click(&context, &mut state, &snapshot, "Review latest");
    assert_eq!(event, Some(Event::Evidence));
}
