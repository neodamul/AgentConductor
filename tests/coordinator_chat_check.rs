#![cfg(unix)]
//! GitHub #326: the workspace snapshot says which agents cannot coordinate chat, so the settings can warn before a question fails.
use agentconductor::ui::setup::Startup;
use serde_json::json;

#[test]
fn the_snapshot_names_agents_that_cannot_answer_chat() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&root, exe).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let custom = |id: &str, read_only: serde_json::Value| {
        json!({"id": id, "executable": "aipro", "args": ["run", "{task_file}"], "parser": "custom",
               "permission_mode": "operator_auto_approve", "read_only_args": read_only})
    };
    let profiles = json!([
        custom("plain", json!(null)),
        custom("declared", json!(["run", "--read-only", "{task_file}"]))
    ]);
    let packet =
        json!({"kind": "save_profiles", "revision": state["revision"], "profiles": profiles});
    startup.dispatch(packet.to_string().as_bytes()).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let blockers = state["chat_blockers"].as_object().unwrap();
    // The reason is in the machine's language; it names the agent and its permission either way.
    let reason = blockers["plain"].as_str().unwrap();
    assert!(
        reason.contains("plain") && reason.contains("operator_auto_approve"),
        "{blockers:?}"
    );
    assert!(!blockers.contains_key("declared"), "{blockers:?}");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_snapshot_flags_automatic_approval_that_nothing_could_show_refused() {
    // GitHub #341.
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&root, exe).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let custom = |id: &str, extra: serde_json::Value| {
        let mut p = json!({"id": id, "executable": "aipro", "args": ["run", "{task_file}"], "parser": "custom",
               "permission_mode": "operator_auto_approve"});
        p.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        p
    };
    let profiles = json!([
        custom("bare", json!({})),
        custom(
            "watched",
            json!({"denial_condition": {"pointer": "/status", "equals": "error"}})
        )
    ]);
    let packet =
        json!({"kind": "save_profiles", "revision": state["revision"], "profiles": profiles});
    startup.dispatch(packet.to_string().as_bytes()).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let hints = state["config_hints"].as_object().unwrap();
    assert_eq!(
        hints["bare"], "startup.auto_approve_unconfigured",
        "{hints:?}"
    );
    assert!(!hints.contains_key("watched"), "{hints:?}");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
