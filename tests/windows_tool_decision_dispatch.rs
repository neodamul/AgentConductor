#[path = "support/windows_tool_decision_diagnostics.rs"]
mod diagnostics;

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
fn receipt_timeout_summary_formats_supported_receipt_fields() {
    use agentconductor::gates::{Context, Receipt, ToolRequest};

    let receipt = Receipt {
        id: "sha".into(),
        context: Context {
            epoch: 1,
            task: "task-1".into(),
            session: "session".into(),
            workspace: "workspace".into(),
            plan_sha256: "plan".into(),
            workspace_sha256: "tree".into(),
        },
        request: ToolRequest {
            tool: "Write".into(),
            input: serde_json::Value::Null,
            reason: "test".into(),
        },
        status: "pending".into(),
        created_at: 1,
        expires_at: None,
    };

    let summary = diagnostics::receipt_summary(Ok(Some(receipt)));
    assert!(summary.contains("id:sha"));
    assert!(summary.contains("status:pending"));
    assert!(summary.contains("task:task-1"));
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
fn checked_activation_journals_uia_and_pointer_results_before_waiting() {
    let source = include_str!("support/windows_tool_decision_script.rs");

    for diagnostic in [
        "ACTION_INVOKE_NO_STATE_CHANGE",
        "$invokeError = $_.Exception.Message",
        "ACTION_POINTER_CLICKED",
        "ACTION_STATE_NOT_VISIBLE_AFTER_POINTER",
    ] {
        assert!(
            source.contains(diagnostic),
            "missing activation diagnostic: {diagnostic}"
        );
    }
    assert!(source.contains("function Find-Button($name)"));
    assert!(source.contains("Find-Button $expected"));
    assert!(source.contains("$clickFailure"));
}

#[test]
fn tool_actions_target_the_modal_and_refresh_dismisses_it_before_card_click() {
    let source = include_str!("support/windows_tool_decision_script.rs");

    let dismiss = source
        .find("if ($action -eq 'refresh') {")
        .expect("refresh must close the blocking pending-request modal");
    let action_search = source
        .find("$button = Find-Button $actionName")
        .expect("decision actions must choose the correct modal or card target");
    assert!(dismiss < action_search);
    assert!(source.contains("function Find-ModalButton($name)"));
    assert!(source.contains("Find-ModalButton $actionName"));
    assert!(source.contains("Find-ModalButton 'Decide later'"));
    assert!(source.contains("Activate $later 'Decide later' $false"));
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

#[test]
fn refresh_timeout_reports_receipt_report_and_controller_action_logs() {
    let source = include_str!("windows_tool_decision.rs");
    let receipt_formatter = include_str!("support/windows_tool_decision_diagnostics.rs");

    for evidence in [
        "refresh_failure_diagnostics",
        "owner.json",
        "stdout.log",
        "stderr.log",
        "workflow::load(run)",
        "ledger.receipt(id)",
    ] {
        assert!(
            source.contains(evidence),
            "missing timeout evidence: {evidence}"
        );
    }
    assert!(receipt_formatter.contains("receipt.status"));
    assert!(!receipt_formatter.contains("receipt={receipt:?}"));
}
