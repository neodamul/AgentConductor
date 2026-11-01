#[test]
fn goal_driver_focuses_and_types_before_saving() {
    let source = include_str!("windows_desktop.rs");
    let focus = source.find("[DesktopInput]::Focus($hwnd, $expectedProcessId)");
    let goal_focus = source.find("$goal.SetFocus()");
    let set_value = source.find("[DesktopInput]::TypeText('Windows UIA goal')");
    let save = source.find("$save.SetFocus()");
    assert!(focus.is_some_and(|index| goal_focus.is_some_and(|goal| index < goal)));
    assert!(goal_focus.is_some_and(|index| set_value.is_some_and(|set| index < set)));
    assert!(set_value.is_some_and(|index| Some(index) < save));
    assert!(source.contains("public static void TypeText(string text)"));
}

#[test]
fn goal_driver_uses_unicode_keyboard_input_and_reads_back_the_value() {
    let source = include_str!("windows_desktop.rs");
    assert!(source.contains("[System.Windows.Automation.ValuePattern]::Pattern"));
    assert!(source.contains("[DesktopInput]::TypeText('Windows UIA goal')"));
    assert!(source.contains("$goalPattern.Current.Value"));
    assert!(source.contains("KEYEVENTF_UNICODE"));
    assert!(source.contains("SetForegroundWindow(window)"));
    assert!(source.contains("[DesktopInput]::PressEnter()"));
}

#[test]
fn goal_driver_reports_the_focused_control_and_exact_goal_value() {
    let source = include_str!("windows_desktop.rs");
    assert!(source.contains("[System.Windows.Automation.AutomationElement]::FocusedElement"));
    assert!(source.contains("[System.Windows.Automation.ValuePattern]::Pattern"));
    assert!(source.contains("GOAL_FOCUS_AFTER_INPUT:"));
    assert!(source.contains("GOAL_TEXT_AFTER_INPUT:"));
    assert!(source.contains("driver output: {report}"));
}

#[test]
fn goal_driver_waits_for_enabled_controls_and_focus_before_typing() {
    let source = include_str!("windows_desktop.rs");
    let wait_enabled = source.find("$goal.Current.IsEnabled -or -not $save.Current.IsEnabled");
    let focus = source.find("[DesktopInput]::Focus($hwnd, $expectedProcessId)");
    let wait_focus = source.find("-not $goal.Current.HasKeyboardFocus");
    let set_value = source.find("[DesktopInput]::TypeText('Windows UIA goal')");
    assert!(wait_enabled.is_some_and(|index| Some(index) < focus));
    assert!(focus.is_some_and(|index| Some(index) < wait_focus));
    assert!(wait_focus.is_some_and(|index| Some(index) < set_value));
}

#[test]
fn goal_driver_types_text_through_unicode_keyboard_input() {
    let source = include_str!("windows_desktop.rs");
    assert!(source.contains("KEYEVENTF_UNICODE = 4"));
    assert!(source.contains("public static void TypeText(string text)"));
    assert!(source.contains("SendInput(2, keys, Marshal.SizeOf(typeof(Input)))"));
    assert!(source.contains("[DesktopInput]::TypeText('Windows UIA goal')"));
}

#[test]
fn goal_driver_types_and_reads_back_text_through_the_uia_value_pattern() {
    let source = include_str!("windows_desktop.rs");
    let pattern =
        source.find("$goal.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)");
    let set_value = source.find("[DesktopInput]::TypeText('Windows UIA goal')");
    let read_value = source.find("$goalPattern.Current.Value");
    let save = source.find("$save.SetFocus()");
    assert!(pattern.is_some());
    assert!(pattern.is_some_and(|index| set_value.is_some_and(|set| index < set)));
    assert!(set_value.is_some_and(|set| read_value.is_some_and(|read| set < read)));
    assert!(read_value.is_some_and(|read| save.is_some_and(|save| read < save)));
}

#[test]
fn save_driver_uses_keyboard_activation_when_invoke_pattern_is_missing() {
    let source = include_str!("windows_desktop.rs");
    let focus = source.find("$save.SetFocus()");
    let wait_focus = source.find("-not $save.Current.HasKeyboardFocus");
    let enter = source.find("[DesktopInput]::PressEnter()");
    assert!(focus.is_some_and(|index| Some(index) < wait_focus));
    assert!(wait_focus.is_some_and(|index| Some(index) < enter));
    assert!(!source.contains("InvokePattern"));
}

#[test]
fn exact_package_includes_the_candidate_review_and_merge_journey() {
    let workflow = include_str!("../.github/workflows/platform-validation.yml");
    let driver = include_str!("support/windows_candidate_review_script.rs");
    assert!(
        workflow.contains("--test windows_candidate_review"),
        "exact Windows package must run the candidate evidence, decision, and merge journey"
    );
    assert!(
        driver.contains("Wait-Text '+implemented'"),
        "exact Windows package must inspect candidate diff content, not only its filename"
    );
    assert!(
        driver.contains("System.Windows.Automation.TextPattern"),
        "exact Windows package must read multiline diff text through UI Automation"
    );
}

#[test]
fn exact_package_runs_the_stop_resume_recovery_journey() {
    let workflow = include_str!("../.github/workflows/platform-validation.yml");

    assert!(workflow.contains("--test package_run_recovery"));
}

#[test]
fn candidate_review_driver_rebinds_the_window_before_reading_controls() {
    let driver = include_str!("support/windows_candidate_review_script.rs").replace("\r\n", "\n");
    assert!(driver.contains("function Find-Window"));
    assert!(driver.contains("function Nodes {\n  $script:window = Find-Window"));
}

#[test]
fn candidate_decision_driver_falls_back_to_owned_input_and_checks_state() {
    let driver = include_str!("support/windows_candidate_review_script.rs");
    assert!(driver.contains("function Activate-Button"));
    assert!(driver.contains("WindowFromPoint"));
    assert!(driver.contains("SendInput(1, new[] { down }, size)"));
    assert!(driver.contains("ACTION_STATE_CONFIRMED_AFTER_POINTER"));
    assert!(driver.contains("Wait-Button 'Merge approved candidate'"));
    assert!(driver.contains("if ($expected -and (Text-Content).Contains($expected))"));
    assert!(!driver.contains("Wait-Text 'Rejected'"));
    assert!(!driver.contains("Wait-Text 'Merged; this run is complete.'"));
    let release = driver.find("SendInput(1, new[] { up }, size)").unwrap();
    let focus_check = driver
        .find("if (!foregroundOwned) return \"FOREGROUND_CHANGED\";")
        .unwrap();
    assert!(
        release < focus_check,
        "mouse-up must be sent before any post-click early return"
    );
    assert!(driver.contains("Activate-Button 'Confirm candidate decision' $null"));
    assert_eq!(
        driver
            .matches("Activate-Button 'Confirm candidate decision' $null")
            .count(),
        3
    );
    assert!(
        include_str!("windows_candidate_review.rs")
            .contains("workflow::load(&run).unwrap().state == expected_state")
    );
}

#[test]
fn candidate_confirmation_accepts_owned_dialog_outside_primary_window_bounds() {
    let driver = include_str!("support/windows_candidate_review_script.rs");
    assert!(driver.contains("if ($bounds.IsEmpty -or $button.Current.IsOffscreen)"));
    assert!(!driver.contains("$bounds.Left -lt $window.Left"));
    assert!(driver.contains("POINT_NOT_OWNED"));
    let owner_check = driver.find("ProcessId(WindowFromPoint").unwrap();
    let pointer_down = driver.find("SendInput(1, new[] { down }, size)").unwrap();
    assert!(owner_check < pointer_down);
}

#[test]
fn rejection_driver_waits_for_an_enabled_reason_field_before_setting_it() {
    let driver = include_str!("support/windows_candidate_review_script.rs");
    let retry = driver.find("while (-not $reasonSet").unwrap();
    let wait = driver
        .find("ControlType.Edit' -and $_.Current.IsEnabled")
        .unwrap();
    let set = driver
        .find(".SetValue('Reviewed exact diff; reject fixture')")
        .unwrap();
    assert!(
        retry < wait && wait < set,
        "the driver must retry only with an enabled reason control before UIA SetValue"
    );
}

#[test]
fn approval_opens_review_changes_before_waiting_for_merge() {
    let driver = include_str!("support/windows_candidate_review_script.rs");
    let approve = driver
        .find("Activate-Button 'Approve verified candidate'")
        .unwrap();
    let merge = approve
        + driver[approve..]
            .find("Wait-Button 'Merge approved candidate'")
            .unwrap();
    assert!(
        driver[approve..merge].contains("Invoke-Button 'Review changes'"),
        "approved candidate must open the review panel before the merge action is available"
    );
}
