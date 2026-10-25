#[test]
fn schedule_driver_scrolls_the_button_into_view_before_activation() {
    let source = include_str!("windows_schedule.rs");
    let focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let scroll = source
        .find("$schedule.GetCurrentPattern([System.Windows.Automation.ScrollItemPattern]::Pattern).ScrollIntoView()")
        .unwrap();
    let geometry = source.find("SCHEDULE_GEOMETRY:").unwrap();
    let dispatch = source.find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()").unwrap();
    assert!(focus < set_time && set_time < scroll && scroll < geometry && geometry < dispatch);
    assert!(source.contains("$schedule = $window.FindAll"));
    assert!(source.contains("while ((Get-Date) -lt $buttonDeadline -and -not $schedule)"));
    assert!(source.contains("$schedule.Current.IsOffscreen"));
    assert!(source.contains("SCHEDULE_ACTION_INVOKED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}
