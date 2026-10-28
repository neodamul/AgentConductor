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

#[test]
fn checked_fallback_brings_offscreen_actions_into_the_window_before_clicking() {
    let source = include_str!("support/windows_tool_decision_script.rs");

    assert!(source.contains("ShowWindow(window, 3)"));
    assert!(source.contains("function Ensure-ButtonInsideWindow"));
    assert!(source.contains("$bounds.Left -ge $windowBounds.Left"));
    assert!(source.contains("[ToolDecisionMouse]::Scroll"));
    assert!(source.contains("BUTTON_OUTSIDE_WINDOW"));
    assert!(source.contains("POINT_NOT_OWNED"));
    let scroll = source
        .split("public static string Scroll")
        .nth(1)
        .and_then(|tail| tail.split("public static string Click").next())
        .expect("scroll helper must stay separately auditable");
    assert!(scroll.contains("if (pointProcessId != expectedPid)"));
}
