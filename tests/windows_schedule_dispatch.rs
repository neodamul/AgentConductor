#[test]
fn schedule_driver_prefers_uia_with_physical_click_fallback() {
    let source = include_str!("windows_schedule.rs");
    assert!(source.contains("$timePattern.SetValue($scheduledAt)"));
    assert!(source.contains(
        "$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()"
    ));
    assert!(source.contains("if (-not [NativeWindowInput]::Click($hwnd, $x, $y))"));
    assert!(source.contains("schedule_support::retry_schedule(||"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(3))"));
}
