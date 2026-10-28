#[test]
fn goal_driver_uses_and_confirms_value_pattern_before_saving() {
    let source = include_str!("windows_desktop.rs");
    assert!(
        !source.contains("$window.SetFocus()"),
        "desktop UI Automation must not focus a non-focusable top-level window"
    );
    let set = source.find("$goalPattern.SetValue('Windows UIA goal')");
    let verify = source.find("while ($goalPattern.Current.Value");
    let save = source.find("$save.GetCurrentPattern(");
    assert!(set.is_some_and(|index| Some(index) < verify));
    assert!(verify.is_some_and(|index| Some(index) < save));
    assert!(!source.contains("SendKeys]::SendWait"));
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
