#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{status_color, workspace::planning_activity_color};

#[test]
fn plan_review_and_run_status_use_a_visible_smooth_pulse() {
    let snapshot = serde_json::json!({
        "job":{"kind":"plan","state":"running","started_ms":0}
    });
    let red = |time| status_color("running", time).r();
    let low = red(0.0);
    let mid = red(0.4);
    let high = red(0.8);
    assert!(
        low < mid && mid < high,
        "pulse should brighten smoothly: {low}, {mid}, {high}"
    );
    assert_eq!(low, red(1.6), "pulse should repeat every 1.6 seconds");

    let plan_low = planning_activity_color(&snapshot, 0).unwrap();
    let plan_high = planning_activity_color(&snapshot, 800).unwrap();
    assert!(plan_high.r() > plan_low.r());
}
