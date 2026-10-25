#[test]
fn schedule_driver_dispatches_with_verified_physical_click() {
    let source = include_str!("windows_schedule.rs");
    assert!(source.contains("$timePattern.SetValue($scheduledAt)"));
    assert!(source.contains("if (-not [NativeWindowInput]::Click($hwnd, $x, $y))"));
    assert!(!source.contains(
        "$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()"
    ));
}
