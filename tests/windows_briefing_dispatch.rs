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
    let find_menu = source
        .find("$menu = Find-Button 'Team briefing · read-only'")
        .unwrap();
    let retry = source.find("if (-not $menu) {").unwrap();
    let click = source.find("[BriefingMouse]::Click($hwnd").unwrap();
    assert!(focus < reacquire && reacquire < invoke);
    assert!(invoke < find_menu && find_menu < retry && retry < click);
    assert!(source.contains("$menu = Find-Button 'Team briefing · read-only'"));
    assert!(source.contains("WindowFromPoint(Point"));
    assert!(source.contains("GetWindowThreadProcessId"));
    assert!(source.contains("mouse_event(0x0001"));
    assert!(source.contains("BRIEFING_TARGET_PROCESS_MISMATCH"));
}
