#[test]
fn goal_driver_uses_and_confirms_value_pattern_before_saving() {
    let source = include_str!("windows_desktop.rs");
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
}
