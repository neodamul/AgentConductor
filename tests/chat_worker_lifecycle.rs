use agentconductor::process;

#[test]
fn chat_jobs_use_background_lifetime_but_other_ui_jobs_remain_owned() {
    assert!(process::is_background_operation("chat"));
    assert!(!process::is_background_operation("mcp"));
}

#[test]
fn chat_worker_uses_maintenance_exit_without_changing_other_cli_commands() {
    assert!(process::is_maintenance_command(&[
        "chat".into(),
        "run-1".into()
    ]));
    assert!(process::is_maintenance_command(&[
        "manual".into(),
        "render".into()
    ]));
    assert!(!process::is_maintenance_command(&[
        "ui".into(),
        "run-1".into()
    ]));
}
