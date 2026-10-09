use agentconductor::{
    adapters::{InputMode, invocation},
    protocol::Provider,
};

#[test]
fn codex_resume_selects_an_exact_session_without_initial_only_flags() {
    let command = invocation(Provider::Codex, Some("thread-123"), "feedback.txt").unwrap();
    assert_eq!(command.executable, "codex");
    assert_eq!(
        command.args,
        ["exec", "resume", "thread-123", "-", "--json"]
    );
    assert_eq!(command.input, InputMode::Stdin);
}

#[test]
fn claude_stream_mode_includes_verbose_and_explicit_resume() {
    let command = invocation(Provider::Claude, Some("session-123"), "feedback.txt").unwrap();
    assert_eq!(command.executable, "claude");
    assert_eq!(
        command.args,
        [
            "-p",
            "--output-format",
            "stream-json",
            "--verbose",
            "--resume",
            "session-123"
        ]
    );
    assert!(!command.args.iter().any(|arg| arg.contains("bypass")));
}

#[test]
fn opencode_uses_a_literal_file_argument_not_shell_interpolation() {
    let path = "task file; echo unsafe.md";
    let command = invocation(Provider::OpenCode, Some("session_123"), path).unwrap();
    assert_eq!(command.input, InputMode::File);
    assert_eq!(command.args[4], path);
    assert!(
        command
            .args
            .windows(2)
            .any(|pair| pair == ["--session", "session_123"])
    );
    assert!(
        !command
            .args
            .iter()
            .any(|arg| arg == "--continue" || arg == "--auto")
    );
}

#[test]
fn invalid_sessions_and_unregistered_custom_profiles_are_rejected() {
    for session in ["", "--last", "id with spaces"] {
        assert!(invocation(Provider::Codex, Some(session), "task.txt").is_err());
    }
    assert!(invocation(Provider::Custom, None, "task.txt").is_err());
}

#[test]
fn initial_codex_execution_has_workspace_write_sandbox() {
    let command = invocation(Provider::Codex, None, "task.txt").unwrap();
    assert_eq!(
        command.args,
        ["exec", "-", "--json", "--sandbox", "workspace-write"]
    );
}
