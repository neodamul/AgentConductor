#[test]
fn exact_package_chat_test_is_wired_into_the_windows_package_gate() {
    let package_test = include_str!("windows_coordinator_chat.rs");
    let workflow = include_str!("../.github/workflows/platform-validation.yml");
    assert!(package_test.contains("AGENTCONDUCTOR_DESKTOP_BINARY"));
    assert!(package_test.contains("chat_transcript"));
    assert!(package_test.contains("in-flight"));
    assert!(workflow.contains("--test windows_coordinator_chat"));
}
