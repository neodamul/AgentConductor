#[test]
fn briefing_driver_retries_with_a_real_click_when_invoke_has_no_effect() {
    let source = include_str!("windows_briefing.rs");
    let invoke = source
        .find(
            "$load.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()",
        )
        .unwrap();
    let find_menu = source
        .find("$menu = Find-Button 'Team briefing · read-only'")
        .unwrap();
    let retry = source.find("if (-not $menu) {").unwrap();
    let click = source.find("[BriefingMouse]::Click($hwnd").unwrap();
    assert!(invoke < find_menu && find_menu < retry && retry < click);
    assert!(source.contains("$menu = Find-Button 'Team briefing · read-only'"));
}
