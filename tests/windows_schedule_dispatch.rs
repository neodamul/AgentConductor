#[test]
fn schedule_driver_maximizes_scrolls_and_targets_the_packaged_schedule_button() {
    let source = include_str!("windows_schedule.rs");
    let maximize = source.find("[ScheduleWindow]::Maximize($hwnd)").unwrap();
    let discover = source.find("$name -eq 'Schedule retry'").unwrap();
    let window_focus = source.find("$window.SetFocus()").unwrap();
    let set_time = source.find("$timePattern.SetValue($scheduledAt)").unwrap();
    let target = source.find("[ScheduleWindow]::ProcessAt($x, $y)").unwrap();
    let reacquire_window = source
        .rfind("$window = $A::RootElement.FindAll($T::Children, $C)")
        .unwrap();
    let reacquire = source
        .find("$schedule = $window.FindAll($T::Descendants, $C)")
        .unwrap();
    let scroll = source
        .find("[ScheduleWindow]::Scroll($scrollX, $scrollY)")
        .unwrap();
    let visible = source.find("SCHEDULE_BUTTON_OUTSIDE_WINDOW").unwrap();
    let focus = visible
        + source[visible..]
            .find("[ScheduleWindow]::Focus($hwnd)")
            .unwrap();
    let post_focus_wait = source[focus..]
        .find("$postFocusDeadline = (Get-Date).AddSeconds(5)")
        .map(|offset| focus + offset)
        .unwrap();
    let uia_refocus = source.rfind("$window.SetFocus()").unwrap();
    let invoke = source.find("$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()").unwrap();
    let submitted = source.find("SCHEDULE_ACTION_SUBMITTED").unwrap();
    assert!(maximize < discover);
    assert!(discover < window_focus && window_focus < set_time);
    assert!(set_time < scroll && scroll < visible);
    assert!(
        visible < focus
            && focus < uia_refocus
            && uia_refocus < post_focus_wait
            && post_focus_wait < reacquire_window
            && reacquire_window < reacquire
            && reacquire < target
            && target < invoke
            && invoke < submitted
    );
    assert!(source.contains("if ($schedule -and $schedule.Current.IsEnabled) { break }"));
    assert!(source.contains("} while ((Get-Date) -lt $postFocusDeadline)"));
    assert!(source.contains("SCHEDULE_AFTER: enabled=$($schedule.Current.IsEnabled)"));
    assert!(source.contains("mouse_event(0x0800, 0, 0, unchecked((uint)-360"));
    assert!(source.contains("ShowWindow(window, 3)"));
    assert!(source.contains("$bounds.Bottom -le $windowBounds.Bottom"));
    assert!(source.contains("$targetPid -ne [uint32]$env:AC_DESKTOP_PID"));
    assert!(!source.contains("[ScheduleWindow]::Click($x, $y)"));
    assert!(source.contains("GetForegroundWindow"));
    assert!(source.contains("GetWindowThreadProcessId"));
    assert!(!source.contains("$time.SetFocus()"));
    assert!(source.contains("SCHEDULE_TARGET_PROCESS_MISMATCH"));
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
