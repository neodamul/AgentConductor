#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::Editor;
use serde_json::{Value, json};

fn profile(id: &str) -> Value {
    json!({"id":id,"parser":"custom","executable":"aipro","args":[],
        "input_mode":"stdin","permission_mode":"read_only","max_sessions":1,
        "quota_refresh_minutes":5})
}

#[test]
fn new_profile_enforces_capacity_and_uniqueness_without_mutating_the_registry() {
    let profiles = vec![profile("company")];
    let mut editor = Editor::default();
    editor.open_new(&profile("company"));
    assert!(!editor.agent_id_is_unique(&profiles));
    assert!(editor.proposed_profiles(&profiles).is_err());
    editor.set_agent_id("helper");
    let proposed = editor.proposed_profiles(&profiles).unwrap();
    assert_eq!(proposed.len(), 2);
    assert_eq!(profiles.len(), 1);
    assert!(
        editor.renames().is_empty(),
        "new registration cannot rename an existing agent"
    );
    let full = (0..16)
        .map(|i| profile(&format!("agent-{i}")))
        .collect::<Vec<_>>();
    assert!(editor.proposed_profiles(&full).is_err());
}

#[test]
fn provider_switch_retains_registration_mode_and_cancel_keeps_existing_profiles() {
    let profiles = vec![profile("company")];
    let mut editor = Editor::default();
    editor.open_new(&profile("helper"));
    let template = json!({"id":"codex","parser":"codex","executable":"codex",
        "args":["exec","-","--json"],"input_mode":"stdin","max_sessions":2});
    editor.switch_provider(&template, &profiles);
    let proposed = editor.proposed_profiles(&profiles).unwrap();
    assert_eq!(proposed[0], profiles[0]);
    assert_eq!(proposed[1]["parser"], "codex");
    assert!(editor.renames().is_empty());
    editor.cancel();
    assert_eq!(editor.proposed_profiles(&profiles).unwrap(), profiles);
}

#[test]
fn edit_cannot_recreate_a_concurrently_removed_agent() {
    let mut editor = Editor::default();
    editor.open(&profile("removed"));
    editor.set_agent_id("renamed");
    assert!(editor.proposed_profiles(&[profile("other")]).is_err());
    assert!(editor.is_editing());
}

#[test]
fn custom_quota_command_and_rules_can_be_saved_from_profile_editor() {
    let mut editor = Editor::default();
    editor.open_new(&profile("company"));
    editor.set_extensions(
        r#"{"quota_command":{"args":["usage"],"timeout_seconds":15},"quota_rules":[{"event_type":"usage","bucket_pointer":"/account","label_pointer":"/period","used_percent_pointer":"/used"}]}"#,
    );

    let saved = editor.proposed_profiles(&[]).unwrap().remove(0);
    assert_eq!(saved["quota_command"]["args"], json!(["usage"]));
    assert_eq!(saved["quota_command"]["timeout_seconds"], 15);
    assert_eq!(saved["quota_rules"][0]["event_type"], "usage");
}
