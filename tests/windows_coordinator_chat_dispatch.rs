#[test]
fn exact_package_chat_test_is_wired_into_the_windows_package_gate() {
    let package_test = include_str!("windows_coordinator_chat.rs");
    let workflow = include_str!("../.github/workflows/platform-validation.yml");
    assert!(package_test.contains("AGENTCONDUCTOR_DESKTOP_BINARY"));
    assert!(package_test.contains("chat_history"));
    assert!(!package_test.contains("chat_transcript"));
    assert!(package_test.contains("in-flight"));
    assert!(package_test.contains("handles_a_slow_coordinator_answer_without_losing_the_chat"));
    assert!(package_test.contains("Some(20_000)"));
    assert!(workflow.contains("--test windows_coordinator_chat"));
    assert!(workflow.contains("--test-threads=1"));
    let script = include_str!("support/windows_chat_script.rs");
    let history = include_str!("support/chat_history_snapshot.rs");
    assert!(history.contains("chat-input-"));
    assert!(history.contains("chat_result"));
    assert!(history.contains("\"turns\""));
    assert!(history.contains("is_ascii_hexdigit"));
    assert!(script.contains("function RefreshWindow()"));
    assert!(script.contains("$env:AC_CHAT_MODE -in @('send', 'verify')"));
    assert!(script.contains("$window = RefreshWindow"));
    assert!(script.contains("function EnabledButton($name)"));
    assert!(script.contains("$edit.SetFocus()"));
    assert!(script.contains("$editPattern.Current.Value -ne $env:AC_CHAT_QUESTION"));
    assert!(script.contains("CHAT_CONTROL_DIAGNOSTIC"));
    assert!(script.contains("CHAT_CONTROL:"));
    assert!(script.contains("CHAT_WINDOW:"));
    assert!(script.contains("CHAT_DESKTOP_WINDOW:"));
    assert!(package_test.contains("Stdio::piped()"));
    assert!(package_test.contains("stderr_report"));
    assert!(package_test.contains("fn send_without_wait(root: &Path"));
    assert!(package_test.matches("diagnostic_state(root)").count() >= 2);
}

#[test]
fn chat_launcher_requires_a_verified_transition_and_process_owned_fallbacks() {
    let script = include_str!("support/windows_chat_script.rs");
    for contract in [
        "CHAT_LAUNCHER_INVOKE_NO_STATE_CHANGE",
        "function WaitChatEdit",
        "$open.SetFocus()",
        "$focused.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID",
        "$focused.Current.Name -eq 'Ask coordinator'",
        "function IsExactButtonAtPoint($button, $point)",
        "$open.GetClickablePoint()",
        "$A::FromPoint($point)",
        "$walker.GetParent($hit)",
        "[ChatMouse]::Position(",
        "[ChatMouse]::Click([uint32]$env:AC_DESKTOP_PID)",
        "ProcessAt(x, y) != expectedPid",
        "SendInput(1, new[] { down }, size)",
        "CHAT_LAUNCHER_NOT_OPEN",
    ] {
        assert!(
            script.contains(contract),
            "missing chat launcher guard: {contract}"
        );
    }
    let click = script
        .split("public static string Click(uint expectedPid)")
        .nth(1)
        .expect("chat launcher pointer fallback must be explicit");
    assert!(
        click
            .find("ProcessAt(point.X, point.Y) != expectedPid")
            .unwrap()
            < click.find("SendInput(1, new[] { down }, size)").unwrap()
    );
    let fallback = script
        .split("$point = $open.GetClickablePoint()")
        .nth(1)
        .unwrap();
    let position = fallback.find("[ChatMouse]::Position(").unwrap();
    let second_hit_test = fallback
        .find("} elseif (IsExactButtonAtPoint $open $point)")
        .unwrap();
    let click_call = fallback.find("[ChatMouse]::Click(").unwrap();
    assert!(position < second_hit_test && second_hit_test < click_call);
}
