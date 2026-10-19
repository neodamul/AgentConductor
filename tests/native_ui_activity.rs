use agentconductor::native_ui_probe::workspace::planning_activity_color;

#[test]
fn coordinator_indicator_pulses_only_while_plan_is_running() {
    let running = serde_json::json!({
        "job":{"kind":"plan","state":"running","started_ms":0}
    });
    let first = planning_activity_color(&running, 0, true).unwrap();
    let next = planning_activity_color(&running, 500, true).unwrap();
    assert_ne!(first, next);
    assert!(
        planning_activity_color(
            &serde_json::json!({"job":{"kind":"plan","state":"completed"}}),
            500,
            true,
        )
        .is_none()
    );
}

#[test]
fn coordinator_indicator_is_static_when_motion_is_disabled() {
    let running = serde_json::json!({"job":{"kind":"plan","state":"running","started_ms":0}});
    assert_eq!(
        planning_activity_color(&running, 0, false),
        planning_activity_color(&running, 500, false)
    );
}
