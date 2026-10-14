//! User decision 2026-10-14: "Selected model is at capacity. Please try a different model" is
//! momentary; a run retries the task after a short wait, at most three times.
use agentconductor::availability;

#[test]
fn a_model_at_capacity_is_a_short_transient_restriction() {
    let message = "Selected model is at capacity. Please try a different model";
    assert!(availability::transient_capacity(message));
    let restriction = availability::detect(&serde_json::Value::Null, message, 1_000).unwrap();
    assert_eq!(restriction.source, "transient_capacity");
    assert_eq!(
        restriction.available_at,
        Some(1_000 + availability::TRANSIENT_RETRY_SECONDS)
    );
    // Ordinary failures stay failures.
    assert!(!availability::transient_capacity(
        "tests failed: 2 assertions"
    ));
    assert!(availability::detect(&serde_json::Value::Null, "syntax error", 1_000).is_none());
}

#[test]
fn the_coordinator_asks_again_after_a_capacity_reply() {
    let source = include_str!("../src/coordinator.rs");
    assert!(source.contains("retries < 3"));
    assert!(source.contains("crate::availability::transient_capacity(&error)"));
}
