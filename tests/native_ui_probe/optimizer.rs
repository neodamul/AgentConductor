use super::optimizer::{Proposal, State, request_for_test as request};
use serde_json::json;

#[test]
fn optimize_request_validates_and_carries_the_unsaved_goal() {
    let snapshot = json!({"revision":"rev-1", "language":"en"});
    let packet = request(&snapshot, "  Make the app faster  ").unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&packet).unwrap(),
        json!({"kind":"optimize_goal", "revision":"rev-1", "goal":"Make the app faster"})
    );
    assert!(request(&snapshot, " ").is_err());
    assert!(request(&json!({}), "goal").is_err());
}

#[test]
fn a_proposal_changes_the_goal_only_after_explicit_acceptance() {
    let mut state = State::default();
    state.set_proposal(Proposal {
        optimized: "Specific goal".into(),
        changes: vec!["Added a success measure".into()],
        questions: vec!["Which platform?".into()],
    });
    assert_eq!(
        state.take_proposal().map(|proposal| proposal.optimized),
        Some("Specific goal".into())
    );
    assert!(state.take_proposal().is_none());
}

#[test]
fn restored_running_optimizer_keeps_its_wait_state() {
    let mut state = State::default();
    state.restore(&json!({
        "config":{"goal":"Restored goal"},
        "job":{"kind":"optimize","id":"opt-7","state":"running","started_ms":1_000}
    }));
    assert!(state.awaiting());
}

#[test]
fn restored_optimizer_displays_elapsed_and_stale_progress() {
    let mut state = State::default();
    state.restore(&json!({
        "config":{"goal":"Restored goal"},
        "job":{"kind":"optimize","id":"opt-7","state":"running","started_ms":1_000}
    }));
    let context = eframe::egui::Context::default();
    let mut output = context.run_ui(
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show(
                "en",
                "Restored goal",
                false,
                Some("Elapsed 2:00 · No new state for 0:08"),
                ui,
            );
        },
    );
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            eframe::egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(text.contains(&"Elapsed 2:00 · No new state for 0:08"));
}
