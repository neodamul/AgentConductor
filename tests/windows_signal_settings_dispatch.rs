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
    assert!(source.contains("Click-ActionControl 'agentmemory preferred'"));
}
