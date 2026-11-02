use agentconductor::native_ui_probe::workspace::profile_tools::Editor;

#[test]
fn profile_rules_editor_saves_agent_task_defaults() {
    let mut profile = serde_json::json!({
        "id":"local", "parser":"claude", "executable":"claude", "args":["-p"],
        "input_mode":"stdin", "permission_mode":"unverified", "catalog":null
    });
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.set_extensions(
        r#"{"agent_settings":{"capabilities":{"task_defaults":{"extract":{"effort":"low"}}}}}"#,
    );
    editor.apply(&mut profile).unwrap();
    assert_eq!(
        profile["agent_settings"]["capabilities"]["task_defaults"]["extract"]["effort"],
        "low"
    );
}

#[test]
fn coordinator_read_only_args_and_model_survive_profile_edit() {
    let mut profile = serde_json::json!({
        "id":"aipro", "parser":"custom", "executable":"aipro",
        "args":["run", "--model", "{model}", "{task_file}"],
        "read_only_args":["chat", "--read-only", "{model}", "{task_file}"],
        "coordinator_model":"local/one", "input_mode":"stdin",
        "permission_mode":"operator_auto_approve",
        "catalog":{"models":["local/one"],"source":"aipro models","queried_at":1,"cli_version":"test"}
    });
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.apply(&mut profile).unwrap();
    assert_eq!(profile["coordinator_model"], "local/one");
    assert_eq!(profile["read_only_args"][1], "--read-only");
}
