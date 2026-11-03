#[test]
fn goal_driver_types_into_the_focused_goal_before_saving() {
    let source = include_str!("windows_desktop.rs");
    let focus = source.find("[DesktopInput]::Focus($hwnd, $expectedProcessId)");
    let click_goal = source.find("[DesktopInput]::Click($goalX, $goalY)");
    let select_all = source.find("[System.Windows.Forms.SendKeys]::SendWait('^a')");
    let type_goal = source.find("[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal')");
    let save = source.find("$saveBounds = $save.Current.BoundingRectangle");
    assert!(focus.is_some_and(|index| click_goal.is_some_and(|value| index < value)));
    assert!(click_goal.is_some_and(|index| select_all.is_some_and(|value| index < value)));
    assert!(select_all.is_some_and(|index| type_goal.is_some_and(|value| index < value)));
    assert!(type_goal.is_some_and(|index| Some(index) < save));
}

#[test]
fn goal_driver_reads_back_the_keyboard_entered_uia_value() {
    let source = include_str!("windows_desktop.rs");
    assert!(source.contains("[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal')"));
    assert!(source.contains("[DesktopInput]::Click($goalX, $goalY)"));
    assert!(!source.contains("$goalValue.SetValue('Windows UIA goal')"));
    assert!(!source.contains("[DesktopInput]::TypeText('Windows UIA goal')"));
    assert!(
        source
            .contains("$goal.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)")
    );
    assert!(source.contains("$goalValue.Current.Value"));
    assert!(source.contains("SetForegroundWindow(window)"));
    assert!(source.contains("[DesktopInput]::Click($saveX, $saveY)"));
}

#[test]
fn goal_driver_reports_the_focused_control_and_exact_goal_value() {
    let source = include_str!("windows_desktop.rs");
    assert!(source.contains("[System.Windows.Automation.AutomationElement]::FocusedElement"));
    assert!(source.contains("[System.Windows.Automation.ValuePattern]::Pattern"));
    assert!(source.contains("GOAL_FOCUS_AFTER_KEYBOARD:"));
    assert!(source.contains("GOAL_FOCUS_AFTER_CLICK:"));
    assert!(source.contains("GOAL_TEXT_AFTER_KEYBOARD:"));
    assert!(source.contains("driver output: {report}"));
}

#[test]
fn goal_driver_checks_enabled_controls_then_validates_keyboard_text_by_readback() {
    let source = include_str!("windows_desktop.rs");
    let wait_enabled = source.find("$goal.Current.IsEnabled -or -not $save.Current.IsEnabled");
    let focus = source.find("[DesktopInput]::Focus($hwnd, $expectedProcessId)");
    let click_goal = source.find("[DesktopInput]::Click($goalX, $goalY)");
    let type_goal = source.find("[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal')");
    assert!(wait_enabled.is_some_and(|index| Some(index) < focus));
    assert!(focus.is_some_and(|index| click_goal.is_some_and(|click| index < click)));
    assert!(click_goal.is_some_and(|click| type_goal.is_some_and(|text| click < text)));
    assert!(!source.contains("HasKeyboardFocus"));
}

#[test]
fn goal_driver_focuses_the_control_before_typing_its_value() {
    let source = include_str!("windows_desktop.rs");
    assert!(source.contains("[DesktopInput]::Click($goalX, $goalY)"));
    assert!(source.contains("[System.Windows.Forms.SendKeys]::SendWait('^a')"));
    assert!(source.contains("[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal')"));
}

#[test]
fn goal_driver_types_and_reads_back_text_through_the_keyboard_and_uia() {
    let source = include_str!("windows_desktop.rs");
    let pattern =
        source.find("$goal.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)");
    let click_goal = source.find("[DesktopInput]::Click($goalX, $goalY)");
    let type_goal = source.find("[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal')");
    let read_value = source.find("$goalValue.Current.Value");
    let wait_for_value = source.find("while ($goalText -ne 'Windows UIA goal'");
    let save = source.find("$saveBounds = $save.Current.BoundingRectangle");
    assert!(pattern.is_some());
    assert!(click_goal.is_some_and(|index| pattern.is_some_and(|pattern| index < pattern)));
    assert!(pattern.is_some_and(|index| type_goal.is_some_and(|value| index < value)));
    assert!(type_goal.is_some_and(|value| wait_for_value.is_some_and(|wait| value < wait)));
    assert!(wait_for_value.is_some_and(|wait| read_value.is_some_and(|read| wait < read)));
    assert!(wait_for_value.is_some_and(|wait| save.is_some_and(|save| wait < save)));
}

#[test]
fn goal_driver_reacquires_save_after_the_edit_control_changes() {
    let source = include_str!("windows_desktop.rs");
    let read_goal = source.find("if ($goalText -ne 'Windows UIA goal')");
    let refresh_anchor = source.find("# Text edits may rebuild the UI Automation tree");
    let refresh_tree = refresh_anchor.and_then(|index| {
        source[index..]
            .find("$all = $window.FindAll($T::Descendants, $C)")
            .map(|offset| index + offset)
    });
    let save_bounds = source.find("$saveBounds = $save.Current.BoundingRectangle");
    assert!(read_goal.is_some_and(|index| refresh_anchor.is_some_and(|next| index < next)));
    assert!(refresh_tree.is_some_and(|index| save_bounds.is_some_and(|next| index < next)));
}

#[test]
fn desktop_app_is_killed_before_driver_failure_can_panic() {
    let source = include_str!("windows_desktop.rs");
    let kill = source.find("let _ = app.kill()");
    let assert_driver = source.find("driver.status.success(),");
    let saved_goal_wait = source.find("while Instant::now() < saved");
    assert!(saved_goal_wait.is_some_and(|wait| kill.is_some_and(|kill| wait < kill)));
    assert!(kill.is_some_and(|index| assert_driver.is_some_and(|assert| index < assert)));
}

#[test]
fn save_driver_invokes_the_save_button_with_a_visible_pointer_fallback() {
    let source = include_str!("windows_desktop.rs");
    let bounds = source.find("$saveBounds = $save.Current.BoundingRectangle");
    let click = source.find("[DesktopInput]::Click($saveX, $saveY)");
    let invoke = source.find("$saveInvoke.Invoke()");
    assert!(source.contains("[System.Windows.Automation.InvokePattern]::Pattern"));
    assert!(bounds.is_some_and(|index| invoke.is_some_and(|invoke| index < invoke)));
    assert!(invoke.is_some_and(|index| click.is_some_and(|click| index < click)));
    assert!(source.contains("$save.Current.IsOffscreen"));
    assert!(source.contains("$saveBounds.IsEmpty"));
    assert!(source.contains("SetCursorPos(int x, int y)"));
    assert!(source.contains("SEND_INPUT_MOUSE_CLICK_FAILED"));
    assert!(!source.contains("PressEnter"));
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
