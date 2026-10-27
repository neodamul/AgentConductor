#[test]
fn powershell_csharp_click_helper_uses_legacy_compiler_syntax() {
    let source = include_str!("windows_signal_settings.rs");

    assert!(source.contains("uint targetPid;"));
    assert!(source.contains("GetWindowThreadProcessId(target, out targetPid);"));
    assert!(!source.contains("out var targetPid"));
}

#[test]
fn transport_choices_are_dispatched_as_radio_controls() {
    let source = include_str!("windows_signal_settings.rs");

    assert!(source.contains("ControlType.RadioButton"));
    assert!(source.contains("Click-ActionControl 'Local Rust'"));
    assert!(source.contains("Activate-ActionControl 'agentmemory preferred'"));
}

#[test]
fn signal_navigation_uses_a_verified_keyboard_fallback() {
    let source = include_str!("windows_signal_settings.rs");
    let activation = source
        .split("function Activate-ActionControl")
        .nth(1)
        .unwrap_or_default()
        .split("Click-ActionControl 'Settings'")
        .next()
        .unwrap_or_default();

    assert!(source.contains("function Activate-ActionControl"));
    assert!(activation.contains("[SignalSettingsMouse]::Focus($hwnd)"));
    assert!(activation.contains("$control.SetFocus()"));
    assert!(activation.contains("$A::FocusedElement"));
    assert!(activation.contains("$focused.Current.ProcessId -eq $expectedProcessId"));
    assert!(activation.contains("$focused.Current.Name -eq $name"));
    assert!(activation.contains("[SignalSettingsMouse]::Key(0x0D)"));
    assert!(activation.contains("if (-not $activated)"));
}

#[test]
fn settings_navigation_waits_for_the_expected_screen_before_falling_back() {
    let source = include_str!("windows_signal_settings.rs");
    let activation = source
        .split("function Activate-ActionControl")
        .nth(1)
        .unwrap_or_default()
        .split("Click-ActionControl 'Settings'")
        .next()
        .unwrap_or_default();

    assert!(activation.contains("$nextName"));
    let wait = activation
        .find("while (-not (Find-ActionControl $nextName)")
        .unwrap();
    assert!(wait < activation.find("if (-not $activated)").unwrap());
}

#[test]
fn missing_signal_control_reports_the_accessibility_tree() {
    let source = include_str!("windows_signal_settings.rs");
    let diagnostic = source
        .split("function Describe-ActionControls {")
        .nth(1)
        .unwrap()
        .split("function Wait-Name")
        .next()
        .unwrap();

    assert!(source.contains("function Describe-ActionControls"));
    assert!(diagnostic.contains("$A::RootElement.FindAll($T::Children, $C)"));
    assert!(diagnostic.contains("$_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID"));
    assert!(source.contains("observed=$($controls -join ' | ')"));
}

#[test]
fn signal_port_uses_real_keyboard_input_in_the_desktop_test() {
    let source = include_str!("windows_signal_settings.rs");

    assert!(source.contains("public static void ReplaceText(string value)"));
    assert!(source.contains("[SignalSettingsMouse]::ReplaceText('7333')"));
    assert!(!source.contains(".SetValue('7333')"));
}

#[test]
fn signal_save_checks_durable_preference_before_visible_dialog_closure() {
    let source = include_str!("windows_signal_settings.rs");
    let save = source.find("Click-ActionControl 'Save'").unwrap();
    assert!(source.contains("$persistDeadline"));
    assert!(source.contains("SIGNAL_PREFERENCE_NOT_PERSISTED"));
    assert!(source.contains("$env:AC_SIGNAL_WORKSPACE"));
    let persist = source.find("$persistDeadline").unwrap();
    let close = source.find("$closeDeadline").unwrap();
    assert!(save < persist && persist < close);
    assert!(source.contains("Find-VisibleActionControl 'Local Rust'"));
}

#[test]
fn signal_dialog_failure_reports_visible_controls_and_saved_file_contents() {
    let source = include_str!("windows_signal_settings.rs");
    assert!(source.contains("function Find-VisibleActionControl"));
    assert!(source.contains(".Current.IsOffscreen"));
    assert!(source.contains(".Current.BoundingRectangle.IsEmpty"));
    assert!(source.contains("SIGNAL_DIALOG_DID_NOT_CLOSE; persisted="));
    assert!(source.contains("Describe-ActionControls"));
    assert!(source.contains("persisted_settings="));
}
