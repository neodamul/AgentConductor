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
    let scroll_into_view = source
        .find("[ScheduleWindow]::SendMessage($hwnd, 0x020A")
        .unwrap();
    let dispatch = source.find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()").unwrap();
    assert!(maximize < discover && discover < focus && focus < set_time);
    assert!(set_time < wait && wait < scroll_into_view);
    assert!(scroll_into_view < geometry && geometry < clipped && clipped < dispatch);
    assert!(source.contains("[ScheduleWindow]::SendMessage($hwnd, 0x020A, $wheel, $position)"));
    assert!(source.contains("$wheel = [IntPtr](-120 -shl 16)"));
    assert!(source.contains("$attempt -lt 10"));
    let workspace = include_str!("../src/native_ui_probe/workspace.rs");
    let room = workspace.find("room::show(").unwrap();
    let room_scroll = workspace[..room]
        .rfind("egui::ScrollArea::vertical()")
        .unwrap();
    assert!(room_scroll < room);
    assert!(source.contains("$schedule = $e"));
    assert!(!source[set_time..].contains("$schedule = $window.FindAll"));
    assert!(source.contains("SCHEDULE_ACTION_INVOKED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}
