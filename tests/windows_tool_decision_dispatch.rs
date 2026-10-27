#[test]
fn checked_fallback_accepts_a_same_process_modal_foreground_window() {
    let source = include_str!("support/windows_tool_decision_script.rs");

    assert!(
        source.contains("GetWindowThreadProcessId(GetForegroundWindow(), out foregroundProcessId)")
    );
    assert!(source.contains("foregroundProcessId == expectedPid"));
    assert!(!source.contains("GetForegroundWindow() != window"));
}
