#[test]
fn checked_fallback_accepts_a_same_process_modal_foreground_window() {
    let source = include_str!("support/windows_tool_decision_script.rs");

    assert!(
        source.contains("GetWindowThreadProcessId(GetForegroundWindow(), out foregroundProcessId)")
    );
    assert!(source.contains("foregroundProcessId == expectedPid"));
    assert!(!source.contains("GetForegroundWindow() != window"));
}

#[test]
fn checked_pointer_failure_reports_the_guard_that_rejected_activation() {
    let source = include_str!("support/windows_tool_decision_script.rs");

    for reason in [
        "FOREGROUND_NOT_OWNED",
        "POINT_NOT_OWNED",
        "CURSOR_POSITION_FAILED",
        "FOREGROUND_CHANGED",
    ] {
        assert!(source.contains(reason));
    }
    assert!(source.contains("CHECKED_POINTER_CLICK_FAILED: $clickFailure"));
}
