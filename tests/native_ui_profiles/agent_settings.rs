use agentconductor::native_ui_probe::workspace::profile_tools::Editor;

#[test]
fn profile_rules_editor_saves_agent_task_defaults() {
    let mut profile = serde_json::json!({
        "id":"local", "parser":"custom", "executable":"aipro", "args":[],
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
