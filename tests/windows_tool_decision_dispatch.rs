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
fn checked_pointer_fallback_uses_sendinput_after_process_ownership_checks() {
    let source = include_str!("support/windows_tool_decision_script.rs");
    let click = source
        .split("public static string Click(")
        .nth(1)
        .and_then(|tail| tail.split("}\n}").next())
        .expect("checked pointer helper must remain easy to audit");

    let ownership_check = click
        .find("pointProcessId != expectedPid")
        .expect("pointer ownership must be checked");
    let down = click
        .find("SendInput(1, new[] { down }, size)")
        .expect("mouse-down must use SendInput");
    let up = click
        .find("SendInput(1, new[] { up }, size)")
        .expect("mouse-up must use SendInput");
    assert!(ownership_check < down && down < up);
}

#[test]
fn pointer_fallback_uses_and_verifies_the_uia_button_click_point() {
    let source = include_str!("support/windows_tool_decision_script.rs");
    let activation = source
        .split("function Activate($button")
        .nth(1)
        .expect("activation helper must be explicit");
    let point = activation.find("$button.GetClickablePoint()").unwrap();
    let hit_test = activation.find("PointHitsButton $x $y $name").unwrap();
    let click = activation
        .find("[ToolDecisionMouse]::Click($hwnd, $x, $y")
        .unwrap();
    assert!(
        point < hit_test && hit_test < click,
        "pointer fallback must target and verify the named accessible button"
    );
    assert!(source.contains("$A::FromPoint($point)"));
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
fn activation_refreshes_the_app_window_before_looking_for_modal_confirmation() {
    let source = include_str!("support/windows_tool_decision_script.rs");
    let finder = source
        .split("function Find-Button($name)")
        .nth(1)
        .and_then(|tail| tail.split("function Find-ModalButton").next())
        .expect("button lookup must remain isolated");

    assert!(finder.contains("$A::RootElement.FindAll($T::Children, $C)"));
    assert!(finder.contains("$_.Current.ProcessId -eq $targetPid"));
}

#[test]
fn activation_uses_a_process_checked_keyboard_fallback_before_pointer_input() {
    let source = include_str!("support/windows_tool_decision_script.rs");
    let activation = source
        .split("function Activate($button")
        .nth(1)
        .expect("activation helper must be explicit");

    assert!(source.contains("public static string Key(ushort key)"));
    let focus = activation.find("$button.SetFocus()").unwrap();
    let focused = activation.find("$focused = $A::FocusedElement").unwrap();
    let owner = activation
        .find("$focused.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID")
        .unwrap();
    let key = activation.find("[ToolDecisionMouse]::Key(0x0D)").unwrap();
    let pointer = activation.find("ACTION_POINTER_CLICKED").unwrap();
    assert!(focus < focused && focused < owner && owner < key && key < pointer);
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
    assert!(source.contains("$button = Find-CurrentButton $name"));
    assert!(source.contains("Activate $later 'Decide later' $false"));
}

#[test]
fn activation_waits_for_a_fresh_enabled_control_before_ui_input() {
    let source = include_str!("support/windows_tool_decision_script.rs");
    let activation = source
        .split("function Activate($button")
        .nth(1)
        .expect("activation helper must be explicit");

    assert!(source.contains("function Find-CurrentButton($name)"));
    assert!(source.contains("function Wait-EnabledButton($name)"));
    assert!(source.contains("$button.Current.IsEnabled"));
    assert!(activation.contains("Wait-EnabledButton $name"));
}

#[test]
fn activation_waits_for_async_state_without_repeating_approval_inputs() {
    let source = include_str!("support/windows_tool_decision_script.rs");
    let activation = source
        .split("function Activate($button")
        .nth(1)
        .expect("activation helper must be explicit");
    assert!(source.contains("function Wait-ActionState($expected, $visible)"));
    assert!(source.contains("$actionStateTimeoutMs = 4000"));
    assert!(!activation.contains("for ($attempt"));
    assert!(!activation.contains("ACTION_RETRYING_AFTER_NO_STATE_CHANGE"));
    assert_eq!(activation.matches("[ToolDecisionMouse]::Click(").count(), 1);
    assert_eq!(activation.matches("[ToolDecisionMouse]::Key(").count(), 1);
    assert!(activation.contains("Wait-ActionState $expected $visible"));
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
