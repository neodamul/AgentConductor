#[test]
fn schedule_driver_keeps_the_original_button_after_editing_the_time() {
    let source = include_str!("windows_schedule.rs");
    let maximize = source
        .find("[ScheduleWindow]::ShowWindow($hwnd, 3)")
        .unwrap();
    let discover = source.find("$name -eq 'Schedule retry'").unwrap();
    let focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let wait = source
        .find("while (-not $schedule.Current.IsEnabled -and (Get-Date) -lt $buttonDeadline)")
        .unwrap();
    let geometry = source.find("SCHEDULE_GEOMETRY:").unwrap();
    let clipped = source.find("SCHEDULE_BUTTON_CLIPPED_BY_WINDOW").unwrap();
    let dispatch = source.find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()").unwrap();
    assert!(maximize < discover && discover < focus && focus < set_time);
    assert!(set_time < wait && wait < geometry && geometry < clipped && clipped < dispatch);
    assert!(source.contains("$schedule = $e"));
    assert!(!source[set_time..].contains("$schedule = $window.FindAll"));
    assert!(source.contains("SCHEDULE_ACTION_INVOKED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}
