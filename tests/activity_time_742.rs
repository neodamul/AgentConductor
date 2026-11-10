#[test]
fn assigned_task_uses_recorded_start_time() {
    let status = std::process::Command::new("node")
        .arg("tests/activity_time_742.cjs")
        .status()
        .expect("node must be available for the activity time test");
    assert!(status.success(), "activity time regression failed");
}
