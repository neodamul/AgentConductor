#[test]
fn schedule_driver_maximizes_scrolls_and_targets_the_packaged_schedule_button() {
    let source = include_str!("support/windows_schedule_script.rs");
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
    assert!(source.contains("$targetPid -ne $expectedProcessId"));
    assert!(!source.contains("[ScheduleWindow]::Click($x, $y)"));
    assert!(source.contains("GetForegroundWindow"));
    assert!(source.contains("GetWindowThreadProcessId"));
    assert!(source.contains("$focused.Current.Name -eq 'Schedule time · ISO 8601 with timezone'"));
    assert!(source.contains("SCHEDULE_TARGET_PROCESS_MISMATCH"));
    let package_test = include_str!("windows_schedule.rs");
    assert!(
        package_test.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))")
    );
    assert!(package_test.contains("#[ignore = \"Runs only against the exact packaged binary.\"]"));
    assert!(
        include_str!("../.github/workflows/platform-validation.yml").contains("--include-ignored")
    );
}

#[test]
fn schedule_test_accepts_only_the_persisted_time_from_the_exact_package() {
    let source = include_str!("support/windows_schedule_script.rs");
    let support = include_str!("support/windows_schedule.rs");
    let package_test = include_str!("windows_schedule.rs");
    assert!(source.contains("SCHEDULE_ACTION_SUBMITTED"));
    assert!(
        package_test.contains("wait_for_manual_schedule(run, expected, Duration::from_secs(10))")
    );
    assert!(support.contains("source == \"manual_schedule\""));
    assert!(source.contains("2099-01-01T09:00:00+09:00"));
}

#[test]
fn unavailable_uia_window_focus_does_not_abort_after_native_foreground_check() {
    let source = include_str!("support/windows_schedule_script.rs");
    let start = source.rfind("try { $window.SetFocus() }").unwrap();
    let end = start + source[start..].find("$postFocusDeadline").unwrap();
    let focus = &source[start..end];

    assert!(focus.contains("try { $window.SetFocus() }"));
    assert!(focus.contains("SCHEDULE_UIA_FOCUS_UNAVAILABLE:"));
    assert!(!focus.contains("exit "));
}

#[test]
fn schedule_uses_control_focus_or_foreground_checked_pointer_input() {
    let source = include_str!("support/windows_schedule_script.rs");
    let time_focus = source.find("$time.SetFocus()").unwrap();
    let tab = source.find("[ScheduleWindow]::Key(0x09)").unwrap();
    let schedule_focus = source
        .find("$focused.Current.Name -eq 'Schedule retry'")
        .unwrap();
    let enter = source.find("[ScheduleWindow]::Key(0x0D)").unwrap();
    let pointer = source
        .find("[ScheduleWindow]::Click($hwnd, $x, $y, $expectedProcessId)")
        .unwrap();

    assert!(source.contains("$schedule.SetFocus()"));
    assert!(source.contains("$buttonFocused = $schedule.Current.HasKeyboardFocus"));
    assert!(source.contains("if ($buttonFocused) {"));
    assert!(
        source.contains("$focused.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit'")
    );
    assert!(source.contains("$focused.Current.Name -eq 'Schedule retry'"));
    assert!(
        source.contains("$focused.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'")
    );
    assert!(time_focus < tab && tab < schedule_focus && schedule_focus < enter && enter < pointer);
    assert!(source.contains("[ScheduleWindow]::Click($hwnd, $x, $y, $expectedProcessId)"));
    assert!(source.contains("SCHEDULE_CLICK_SENT; method=foreground-checked-pointer"));
    assert!(source.contains("SCHEDULE_CLICK_SENT; method=keyboard"));
    assert!(source.contains("SCHEDULE_KEYBOARD_TARGET_UNAVAILABLE"));
    assert!(source.contains("mouse_event(0x0001, 1, 0"));
    assert!(source.contains("mouse_event(0x0002, 0, 0"));
    assert!(source.contains("mouse_event(0x0004, 0, 0"));
}

#[test]
fn schedule_tries_uia_invoke_before_unchecked_pointer_fallback() {
    let source = include_str!("support/windows_schedule_script.rs");
    let keyboard = source.find("SCHEDULE_CLICK_SENT; method=keyboard").unwrap();
    let invoke = source
        .find("SCHEDULE_CLICK_SENT; method=uia-invoke-fallback")
        .unwrap();
    let pointer = source
        .find("SCHEDULE_CLICK_SENT; method=foreground-checked-pointer")
        .unwrap();
    let fallback = &source[keyboard..pointer];

    assert!(keyboard < invoke && invoke < pointer);
    assert!(fallback.contains("if (-not $keyboardPressed)"));
    assert!(fallback.contains(
        "$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()"
    ));
}

#[test]
fn schedule_falls_back_to_pointer_when_uia_invoke_leaves_button_enabled() {
    let source = include_str!("support/windows_schedule_script.rs");
    let no_state_change = source.find("SCHEDULE_UIA_INVOKE_NO_STATE_CHANGE").unwrap();
    let pointer = source
        .find("[ScheduleWindow]::Click($hwnd, $x, $y, $expectedProcessId)")
        .unwrap();

    assert!(no_state_change < pointer);
    assert!(source.contains("if (Test-ScheduleActionAccepted) {"));
    assert!(source.contains("if ($currentButton -and -not $currentButton.Current.IsEnabled)"));
    assert!(source.contains("$buttonFocused = $false"));
}
