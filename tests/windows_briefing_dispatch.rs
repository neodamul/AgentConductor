#[test]
fn briefing_driver_retries_with_a_real_click_when_invoke_has_no_effect() {
    let source = include_str!("windows_briefing.rs");
    let invoke = source
        .find(
            "$load.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()",
        )
        .unwrap();
    let focus = source.find("[BriefingMouse]::Focus($hwnd)").unwrap();
    let reacquire = source
        .rfind("$window = $A::RootElement.FindAll($T::Children, $C)")
        .unwrap();
    let state_wait = source.find("$activationState = if ($invoked)").unwrap();
    let retry = source
        .find("for ($attempt = 1; $attempt -le 3; $attempt++)")
        .unwrap();
    let retry_button = source[retry..]
        .find("$load = Find-Button 'Load read-only team briefing'")
        .unwrap()
        + retry;
    let click = source.find("[BriefingMouse]::Click($hwnd").unwrap();
    assert!(focus < reacquire && reacquire < invoke && invoke < state_wait);
    assert!(state_wait < retry && retry < retry_button && retry_button < click);
    assert!(source.contains("$menu = Find-Button 'Team briefing · read-only'"));
    assert!(source.contains("WindowFromPoint(Point"));
    assert!(source.contains("GetWindowThreadProcessId"));
    assert!(source.contains("mouse_event(0x0001"));
    assert!(source.contains("BRIEFING_TARGET_PROCESS_MISMATCH"));
    assert!(source.contains("function Wait-BriefingChange"));
    assert!(source.contains("for ($attempt = 1; $attempt -le 3; $attempt++)"));
    assert!(source.contains("$activationState = Wait-BriefingChange"));
    assert!(source.contains("$activationState -eq 'unchanged'"));
    assert!(source.contains("BRIEFING_ACTION_NO_STATE_CHANGE"));
}

#[test]
fn briefing_driver_checks_state_after_each_delivered_pointer_click() {
    let source = include_str!("windows_briefing.rs");
    let retry = source
        .find("for ($attempt = 1; $attempt -le 3; $attempt++)")
        .unwrap();
    let click = source
        .find("if (-not [BriefingMouse]::Click($hwnd")
        .unwrap();
    let verify = source
        .find("$activationState = Wait-BriefingChange")
        .unwrap();

    assert!(retry < click && click < verify);
    assert!(source.contains("WINDOW_FOREGROUND_FAILED_BEFORE_BRIEFING_CLICK"));
    assert!(source.contains("BRIEFING_ACTION_NO_STATE_CHANGE"));
}

#[test]
fn briefing_driver_has_keyboard_fallback_after_pointer_delivery() {
    let source = include_str!("windows_briefing.rs");
    assert!(source.contains("$load.SetFocus()"));
    assert!(source.contains("keybd_event"));
    assert!(source.contains("BRIEFING_KEYBOARD_FALLBACK"));
}
