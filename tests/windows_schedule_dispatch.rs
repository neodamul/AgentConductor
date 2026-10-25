#[test]
fn schedule_driver_prefers_uia_with_physical_click_fallback() {
    let source = include_str!("windows_schedule.rs");
    assert!(source.contains("$timePattern.SetValue($scheduledAt)"));
    assert!(source.contains(
        "$targetSchedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()"
    ));
    assert!(source.contains("$scheduleBeforeEdit = $schedule"));
    assert!(source.contains("$window.SetFocus()"));
    assert!(source.find("$window.SetFocus()") < source.find("$timePattern.SetValue($scheduledAt)"));
    assert!(source.contains("AC_SCHEDULE_DISPATCH_MODE"));
    assert!(source.contains("[\"pre-edit\", \"fresh\", \"mouse\"][number - 1]"));
    assert!(source.contains("schedule_support::retry_schedule(|number|"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(3))"));
}
