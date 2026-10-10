use std::process::Command;

#[test]
fn displays_help_without_starting_agents() {
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("validate"));
}

#[test]
fn validates_a_plan_and_reports_the_default_workspace_mode() {
    let path = std::env::temp_dir().join(format!("ac-cli-plan-{}.json", std::process::id()));
    let plan = serde_json::json!({
        "goal":"refresh token rotation",
        "agents":[{"id":"custom","executable":"aipro","args":[]}],
        "tasks":(["src/auth","tests/auth","docs/auth"].map(|path| serde_json::json!({
            "id":path,"agent":"custom","prompt":"Do the assigned task","paths":[path],
            "test_command":["cargo","test"]
        })))
    });
    std::fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("validate")
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["workspace_mode"], "shared");
    assert_eq!(result["tasks"], 3);
}

#[test]
fn invalid_input_and_unknown_commands_fail_without_panicking() {
    for args in [
        vec!["validate"],
        vec!["missing"],
        vec!["ui-serve", "/nonexistent/run"],
        vec!["validate", "/nonexistent/ac-plan.json"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    }
}

#[test]
fn normalizes_codex_events_without_starting_a_provider() {
    let path = std::env::temp_dir().join(format!("ac-cli-events-{}.jsonl", std::process::id()));
    std::fs::write(&path, b"{\"type\":\"thread.started\",\"thread_id\":\"thread-1\"}\n{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":12}}\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["normalize", "codex"])
        .arg(&path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let events: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(events.len(), 2);
    assert_eq!(events[1]["kind"], "result");
    assert_eq!(events[1]["session_id"], "thread-1");
}
