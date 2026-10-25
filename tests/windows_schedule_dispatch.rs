#[test]
fn schedule_driver_invokes_the_refreshed_button_and_checks_persisted_state() {
    let source = include_str!("windows_schedule.rs");
    let focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let dispatch = source.find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()").unwrap();
    assert!(focus < set_time && set_time < dispatch);
    assert!(source.contains("$schedule = $window.FindAll"));
    assert!(source.contains("while ((Get-Date) -lt $buttonDeadline -and -not $schedule)"));
    assert!(source.contains("SCHEDULE_ACTION_INVOKED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}
