use agentconductor::native_ui_probe::workspace::{execution_ready, pending_message};

#[test]
fn pending_review_start_is_distinguished_from_workspace_saves() {
    assert_eq!(pending_message("start"), "Starting agent execution…");
    assert_ne!(pending_message("start"), pending_message("save"));
}

#[test]
fn pending_review_refresh_reports_plan_progress_when_planning() {
    assert_eq!(pending_message("plan_refresh"), "Refreshing plan progress…");
    assert_eq!(pending_message("save"), "Saving workspace…");
}

#[test]
fn execution_transition_waits_for_the_exact_approved_run_report() {
    let previous_run = serde_json::json!({
        "run_available":true,
        "proposal":{"run":"/runs/previous"}
    });
    assert!(!execution_ready(&previous_run, "/runs/current"));
    let current_not_ready = serde_json::json!({
        "run_available":false,
        "proposal":{"run":"/runs/current"}
    });
    assert!(!execution_ready(&current_not_ready, "/runs/current"));
    let current_ready = serde_json::json!({
        "run_available":true,
        "proposal":{"run":"/runs/current"}
    });
    assert!(execution_ready(&current_ready, "/runs/current"));
}
