#[test]
fn schedule_driver_uses_the_passing_uia_button_invocation() {
    let source = include_str!("windows_schedule.rs");
    let discover = source.find("$name -eq 'Schedule retry'").unwrap();
    let window_focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let focus = source.find("[ScheduleWindow]::Focus($hwnd)").unwrap();
    let target = source.find("[ScheduleWindow]::ProcessAt($x, $y)").unwrap();
    let invoke = source
        .find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()")
        .unwrap();
    let post_action = source.find("POST_ACTION:").unwrap();
    let submitted = source.find("SCHEDULE_ACTION_SUBMITTED").unwrap();
    assert!(discover < window_focus && window_focus < set_time);
    assert!(set_time < focus && focus < target && target < invoke);
    assert!(invoke < post_action && post_action < submitted);
    assert!(!source.contains("[ScheduleWindow]::mouse_event"));
    assert!(source.contains("GetForegroundWindow"));
    assert!(source.contains("GetWindowThreadProcessId"));
    assert!(!source.contains("$time.SetFocus()"));
    assert!(source.contains("SCHEDULE_TARGET_PROCESS_MISMATCH"));
    assert!(source.contains("$schedule = $e"));
    assert!(!source[set_time..].contains("$schedule = $window.FindAll"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}

#[test]
fn schedule_test_accepts_only_the_persisted_time_from_the_exact_package() {
    let source = include_str!("windows_schedule.rs");
    let support = include_str!("support/windows_schedule.rs");
    assert!(source.contains("SCHEDULE_ACTION_SUBMITTED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(support.contains("source == \"manual_schedule\""));
    assert!(source.contains("2099-01-01T09:00:00+09:00"));
}
