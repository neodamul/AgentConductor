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
    let place_pointer = source
        .find("[ScheduleWindow]::SetCursorPos($scrollX, $scrollY)")
        .unwrap();
    let scroll_into_view = source.find("[ScheduleWindow]::mouse_event(0x0800").unwrap();
    let dispatch = source
        .find("[ScheduleWindow]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)")
        .unwrap();
    let release = source
        .find("[ScheduleWindow]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)")
        .unwrap();
    let click_pointer = source
        .find("[ScheduleWindow]::SetCursorPos($clickX, $clickY)")
        .unwrap();
    let pointer_settle = source[click_pointer..]
        .find("Start-Sleep -Milliseconds 150")
        .unwrap()
        + click_pointer;
    let restore_focus = source[click_pointer..]
        .find("[ScheduleWindow]::SetForegroundWindow($hwnd)")
        .unwrap()
        + click_pointer;
    let target_check = source.find("SCHEDULE_TARGET:").unwrap();
    let press_settle = source[dispatch..]
        .find("Start-Sleep -Milliseconds 100")
        .unwrap()
        + dispatch;
    assert!(maximize < discover && discover < focus && focus < set_time);
    assert!(set_time < wait && wait < scroll_into_view);
    assert!(place_pointer < scroll_into_view && scroll_into_view < geometry);
    assert!(geometry < clipped && clipped < click_pointer);
    assert!(click_pointer < pointer_settle && pointer_settle < restore_focus);
    assert!(restore_focus < target_check && target_check < dispatch);
    assert!(source.contains("WindowFromPoint"));
    assert!(source.contains("foreground_pid="));
    assert!(source.contains("target_pid="));
    assert!(dispatch < press_settle && press_settle < release);
    assert!(source.contains("[ScheduleWindow]::mouse_event(0x0800, 0, 0, -360, [UIntPtr]::Zero)"));
    assert!(source.contains("$attempt -lt 10"));
    let workspace = include_str!("../src/native_ui_probe/workspace.rs");
    let room = workspace.find("room::show(").unwrap();
    let room_scroll = workspace[..room]
        .rfind("egui::ScrollArea::vertical()")
        .unwrap();
    assert!(room_scroll < room);
    assert!(source.contains("$schedule = $e"));
    assert!(!source[set_time..].contains("$schedule = $window.FindAll"));
    assert!(source.contains("SCHEDULE_ACTION_SUBMITTED"));
    assert!(source.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))"));
    assert!(source.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}

#[test]
fn schedule_driver_targets_the_inspector_when_scrolling_the_schedule_into_view() {
    let source = include_str!("windows_schedule.rs");
    let wheel = source.find("[ScheduleWindow]::mouse_event(0x0800").unwrap();
    let pointer = source
        .find("[ScheduleWindow]::SetCursorPos($scrollX, $scrollY)")
        .unwrap();
    assert!(source.contains("$scrollX = [int]($windowBounds.Right - 40)"));
    assert!(source.contains("$scrollY = [int]($windowBounds.Bottom - 64)"));
    assert!(
        pointer
            > source
                .find("$scrollY = [int]($windowBounds.Bottom - 64)")
                .unwrap()
    );
    assert!(pointer < wheel);
    assert!(source.contains("mouse_event(0x0800, 0, 0, -360"));
    assert!(!source.contains("SendMessage($hwnd, 0x020A"));
    assert!(!source.contains("RangeValuePattern"));
}
