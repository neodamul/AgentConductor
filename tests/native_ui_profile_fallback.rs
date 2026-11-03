#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{Editor, Event};
use serde_json::json;

#[test]
fn profile_editor_saves_renamed_id_and_fallback_without_losing_private_environment() {
    let source = json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":["-p"],
        "input_mode":"stdin", "permission_mode":"full_access", "max_sessions":2,
        "quota_refresh_minutes":5, "environment":{"TEAM_TOKEN":"private"},
        "catalog":{"models":["sonnet"],"source":"cli","queried_at":1,"cli_version":"1"}
    });
    let mut editor = Editor::default();
    editor.open(&source);
    editor.set_agent_id("claude-secondary");
    editor.set_fallback(Some("codex"), Some("gpt-5-codex"));
    let mut saved = source.clone();
    editor.apply(&mut saved).unwrap();

    assert_eq!(saved["id"], "claude-secondary");
    assert_eq!(
        saved["fallback"],
        json!({"agent":"codex","model":"gpt-5-codex"})
    );
    assert_eq!(saved["environment"]["TEAM_TOKEN"], "private");
    assert_eq!(editor.renames()["claude"], "claude-secondary");

    editor.set_fallback(None, None);
    editor.apply(&mut saved).unwrap();
    assert!(saved["fallback"].is_null());
    editor.set_agent_id("  ");
    assert!(editor.apply(&mut saved).is_err());
}

#[test]
fn provider_switch_preserves_non_provider_fallback_and_environment() {
    let source = json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":["-p"],
        "input_mode":"stdin", "max_sessions":2, "quota_refresh_minutes":10,
        "environment":{"TEAM_TOKEN":"private"},
        "fallback":{"agent":"company","model":"internal-v2"},
        "catalog":{"models":["sonnet"],"source":"cli","queried_at":1,"cli_version":"1"}
    });
    let codex = json!({
        "id":"codex", "parser":"codex", "executable":"codex", "args":["exec"],
        "input_mode":"file", "permission_mode":"native_sandbox", "max_sessions":1,
        "quota_refresh_minutes":5, "catalog":null
    });
    let templates = vec![source.clone(), codex.clone()];
    let mut editor = Editor::default();
    editor.open(&source);
    editor.switch_provider(&codex, &templates);
    let mut saved = source.clone();
    editor.apply(&mut saved).unwrap();

    assert_eq!(saved["id"], "codex-2");
    assert_eq!(saved["parser"], "codex");
    assert_eq!(saved["fallback"], source["fallback"]);
    assert_eq!(saved["environment"], source["environment"]);
    assert!(saved["catalog"].is_null());
}

#[test]
fn fallback_catalog_is_requested_once_only_for_a_saved_builtin_agent() {
    let source = json!({"id":"claude","parser":"claude","executable":"claude","args":[]});
    let codex = json!({"id":"codex","parser":"codex","catalog":null});
    let custom = json!({"id":"company","parser":"custom","catalog":null});
    let mut editor = Editor::default();
    editor.open(&source);
    editor.set_fallback(Some("codex"), None);
    assert_eq!(
        editor.fallback_catalog_event(
            &[source.clone(), codex.clone()],
            std::slice::from_ref(&codex)
        ),
        Some(Event::Discover("codex".into()))
    );
    assert_eq!(
        editor.fallback_catalog_event(
            &[source.clone(), codex.clone()],
            std::slice::from_ref(&codex)
        ),
        None
    );
    editor.set_fallback(Some("company"), None);
    assert_eq!(
        editor.fallback_catalog_event(
            &[source.clone(), custom.clone()],
            std::slice::from_ref(&custom)
        ),
        None
    );
    editor.set_fallback(Some("codex"), None);
    assert_eq!(
        editor.fallback_catalog_event(&[source, codex.clone()], &[]),
        None
    );
}

#[test]
fn agent_id_uniqueness_tracks_a_provider_rename_without_blocking_retry() {
    let source = json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":[],
        "input_mode":"stdin", "max_sessions":1, "quota_refresh_minutes":5
    });
    let codex = json!({"id":"codex","parser":"codex","executable":"codex"});
    let mut profiles = vec![source.clone(), codex.clone()];
    let mut editor = Editor::default();
    editor.open(&source);
    assert!(editor.agent_id_is_unique(&profiles));
    editor.set_agent_id("codex");
    assert!(!editor.agent_id_is_unique(&profiles));
    editor.set_agent_id("claude-reviewer");
    assert!(editor.agent_id_is_unique(&profiles));
    editor.apply(&mut profiles[0]).unwrap();
    assert!(editor.agent_id_is_unique(&profiles));
    assert_eq!(profiles[0]["id"], "claude-reviewer");
}
