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
