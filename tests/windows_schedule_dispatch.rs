#[test]
fn schedule_driver_uses_foreground_mouse_input_and_checks_persisted_state() {
    let source = include_str!("windows_schedule.rs");
    let focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let dispatch = source.find("ScheduleMouse]::Click").unwrap();
    assert!(focus < set_time && set_time < dispatch);
    assert!(source.contains("SetForegroundWindow(window)"));
    assert!(source.contains("SendInput(1, new[] { down }, size)"));
    assert!(source.contains("SCHEDULE_MOUSE_CLICK_SENT"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}
