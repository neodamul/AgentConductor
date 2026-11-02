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
    assert!(
        blockers["plain"]
            .as_str()
            .unwrap()
            .contains("read-only arguments"),
        "{blockers:?}"
    );
    assert!(!blockers.contains_key("declared"), "{blockers:?}");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
