#[test]
fn schedule_driver_uses_the_passing_uia_button_invocation() {
    let source = include_str!("windows_schedule.rs");
    let discover = source.find("$name -eq 'Schedule retry'").unwrap();
    let focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let invoke = source
        .find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()")
        .unwrap();
    let post_action = source.find("POST_ACTION:").unwrap();
    let submitted = source.find("SCHEDULE_ACTION_SUBMITTED").unwrap();
    assert!(discover < focus && focus < set_time && set_time < invoke);
    assert!(invoke < post_action && post_action < submitted);
    assert!(!source.contains("[ScheduleWindow]::mouse_event"));
    assert!(!source.contains("WindowFromPoint"));
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
