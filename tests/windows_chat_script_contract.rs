#[path = "support/windows_chat_script.rs"]
mod script;

#[test]
fn launcher_selection_skips_hidden_and_disabled_buttons() {
    let buttons = script::SCRIPT
        .split("function Buttons($name) {")
        .nth(1)
        .and_then(|tail| tail.split("\n}").next())
        .expect("button selector in UI automation script");

    for condition in [
        "Current.IsEnabled",
        "-not $_.Current.IsOffscreen",
        "-not $_.Current.BoundingRectangle.IsEmpty",
        "Current.ProcessId -eq [int]$env:AC_DESKTOP_PID",
        "if (-not $window) { return $null }",
    ] {
        assert!(
            buttons.contains(condition),
            "missing selector condition: {condition}"
        );
    }
}

#[test]
fn chat_edit_lookup_handles_a_window_that_is_not_ready_yet() {
    let chat_edit = script::SCRIPT
        .split("function ChatEdit(")
        .nth(1)
        .and_then(|tail| tail.split(") {\n").nth(1))
        .and_then(|tail| tail.split("\n}").next())
        .expect("chat editor selector in UI automation script");

    assert!(chat_edit.contains("if (-not $window) { return $null }"));
}
