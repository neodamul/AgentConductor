#[test]
fn signal_control_lookup_reacquires_the_process_owned_top_level_window() {
    let source = include_str!("support/windows_signal_settings_script.rs");
    assert!(source.contains("function Get-ProcessWindows"));
    for function in [
        "Find-Name",
        "Find-ActionControl",
        "Find-VisibleActionControl",
        "Find-Edit",
    ] {
        let definition = format!("function {function}(");
        let body = source
            .split(&definition)
            .nth(1)
            .unwrap_or_default()
            .split("\nfunction ")
            .next()
            .unwrap_or_default();
        assert!(body.contains("Get-ProcessWindows"), "{function}");
        assert!(
            body.contains("$script:window = $candidateWindow"),
            "{function}"
        );
    }
}
