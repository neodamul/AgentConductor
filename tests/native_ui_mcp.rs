use agentconductor::native_ui_probe::workspace::mcp::{save_request, settings_text};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

#[test]
fn native_mcp_editor_loads_server_definitions_and_goal_sources() {
    let snapshot = json!({
        "config": {
            "mcp": {"company": {"type": "local", "command": ["company-mcp"]}},
            "goal_sources": {"jira": {"server": "company", "tool": "get_issue", "arguments": {"id": "{id}"}, "read_only": true}}
        }
    });

    let settings: Value = serde_json::from_str(&settings_text(&snapshot)).unwrap();

    assert_eq!(settings["mcp"]["company"]["command"][0], "company-mcp");
    assert_eq!(settings["goal_sources"]["jira"]["tool"], "get_issue");
}

#[test]
fn native_mcp_editor_starts_with_empty_opencode_shape() {
    let settings: Value = serde_json::from_str(&settings_text(&json!({"config": {}}))).unwrap();

    assert_eq!(settings, json!({"mcp": {}, "goal_sources": {}}));
}

#[test]
fn native_mcp_save_request_uses_current_revision_and_preserves_jsonc_text() {
    let text = "{\n  // company tools\n  \"mcp\": {},\n  \"goal_sources\": {}\n}";
    let packet = save_request(&json!({"revision": "rev-7"}), text).unwrap();
    let request: Value = serde_json::from_slice(&packet).unwrap();

    assert_eq!(request["kind"], "mcp_save");
    assert_eq!(request["revision"], "rev-7");
    assert_eq!(request["settings"], text);
}

#[test]
fn native_mcp_save_request_rejects_packets_over_the_dispatch_limit() {
    let text = " ".repeat(64 * 1024);

    assert!(save_request(&json!({"revision": "rev-7"}), &text).is_err());
}

#[test]
fn native_mcp_settings_save_through_the_async_rust_worker_without_starting_a_server() {
    let root = std::env::temp_dir().join(format!(
        "ac-native-mcp-{}-{}",
        std::process::id(),
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let room = Arc::new(
        agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
    );
    let worker = agentconductor::native_ui_probe::RequestWorker::new(
        room.clone(),
        eframe::egui::Context::default(),
    )
    .unwrap();
    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    let snapshot = worker
        .replies
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();
    let text = r#"{"mcp":{"company":{"type":"local","command":["company-mcp"],"environment":{"TOKEN":"{env:COMPANY_TOKEN}"}}},"goal_sources":{}}"#;

    worker
        .send(&save_request(&snapshot, text).unwrap())
        .unwrap();
    let saved = worker
        .replies
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();

    assert_eq!(saved["saved"], true);
    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    let refreshed = worker
        .replies
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();
    assert_eq!(
        refreshed["config"]["mcp"]["company"]["environment"]["TOKEN"],
        "{env:COMPANY_TOKEN}"
    );
    assert!(refreshed["job"].is_null());

    drop(worker);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    std::fs::remove_dir_all(root).unwrap();
}
