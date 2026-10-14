//! "What should the team work on?" showed "Started" forever after the run finished; the label
//! now follows the run's state.
const STARTUP: &str = include_str!("../src/ui/startup.js");

#[test]
fn a_started_plan_shows_its_run_state() {
    assert!(STARTUP.contains("state?.proposal?.phase==='starting'?runPhaseLabel(state.run_state)"));
    assert!(STARTUP.contains("function runPhaseLabel(runState)"));
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../src/ui/messages/ko.json")).unwrap();
    for state in [
        "running",
        "waiting_approval",
        "approved",
        "completed",
        "failed",
        "stopped",
        "waiting_availability",
    ] {
        assert!(catalog[format!("state.{state}")].is_string(), "{state}");
    }
}
