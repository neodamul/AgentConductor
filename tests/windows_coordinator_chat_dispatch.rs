#[test]
fn exact_package_chat_test_is_wired_into_the_windows_package_gate() {
    let package_test = include_str!("windows_coordinator_chat.rs");
    let workflow = include_str!("../.github/workflows/platform-validation.yml");
    assert!(package_test.contains("AGENTCONDUCTOR_DESKTOP_BINARY"));
    assert!(package_test.contains("chat_transcript"));
    assert!(package_test.contains("in-flight"));
    assert!(workflow.contains("--test windows_coordinator_chat"));
    assert!(workflow.contains("--test-threads=1"));
    let script = include_str!("support/windows_chat_script.rs");
    assert!(script.contains("function EnabledButton($name)"));
    assert!(script.contains("$edit.SetFocus()"));
    assert!(script.contains("$editPattern.Current.Value -ne $env:AC_CHAT_QUESTION"));
    assert!(script.contains("CHAT_CONTROL_DIAGNOSTIC"));
    assert!(script.contains("CHAT_CONTROL:"));
    assert!(script.contains("CHAT_WINDOW:"));
    assert!(script.contains("CHAT_DESKTOP_WINDOW:"));
    assert!(package_test.contains("Stdio::piped()"));
    assert!(package_test.contains("stderr_report"));
}
