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
