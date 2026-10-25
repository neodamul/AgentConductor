#[test]
fn schedule_driver_keeps_the_original_button_after_editing_the_time() {
    let source = include_str!("windows_schedule.rs");
    let focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let wait = source
        .find("while (-not $schedule.Current.IsEnabled -and (Get-Date) -lt $buttonDeadline)")
        .unwrap();
    let geometry = source.find("SCHEDULE_GEOMETRY:").unwrap();
    let dispatch = source.find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()").unwrap();
    assert!(focus < set_time && set_time < wait && wait < geometry && geometry < dispatch);
    assert!(source.contains("$schedule = $e"));
    assert!(!source[set_time..].contains("$schedule = $window.FindAll"));
    assert!(source.contains("SCHEDULE_ACTION_INVOKED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}
