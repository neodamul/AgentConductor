#![cfg(feature = "native-ui-probe")]

use agentconductor::{
    mcp::Settings,
    native_ui_probe::workspace::mcp::{State, discover_request, save_request, settings_text},
};
use eframe::egui;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

#[test]
fn native_mcp_dialog_matches_the_settings_hierarchy_in_both_languages() {
    let snapshot = json!({"config": {}});
    for language in ["en", "ko"] {
        let text = |key| agentconductor::ui::messages::in_language(language, key);
        let context = egui::Context::default();
        let mut editor = State::default();
        editor.open(Some(&snapshot));
        let mut first = context.run_ui(Default::default(), |ui| {
            editor.show(Some(&snapshot), language, true, ui);
        });
        first.textures_delta.set.clear();
        first.textures_delta.free.clear();
        let mut output = context.run_ui(Default::default(), |ui| {
            editor.show(Some(&snapshot), language, true, ui);
        });
        let visible = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ");
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();

        assert!(
            visible.contains(text("mcp.settings_mcp_servers")),
            "{visible}"
        );
        assert!(
            visible.contains(text("mcp.register_mcp_servers")),
            "{visible}"
        );
        assert_ne!(
            text("mcp.mcp_server_registration"),
            text("mcp.register_mcp_servers")
        );
    }
}

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
    assert!(!settings_text(&snapshot).contains("your-mcp-server"));
}

#[test]
fn native_mcp_editor_starts_with_empty_opencode_shape() {
    let settings = Settings::parse(&settings_text(&json!({"config": {}}))).unwrap();

    assert!(settings.mcp.is_empty());
    assert!(settings.goal_sources.is_empty());
}

#[test]
fn native_mcp_editor_shows_inactive_opencode_local_and_remote_examples() {
    let text = settings_text(&json!({"config": {}}));
    let settings = Settings::parse(&text).unwrap();

    for sample in [
        "\"type\": \"local\"",
        "\"command\": [\"your-mcp-server\", \"--stdio\"]",
        "\"environment\": { \"COMPANY_TOKEN\": \"{env:COMPANY_MCP_TOKEN}\" }",
        "\"type\": \"remote\"",
        "https://mcp.example.invalid/mcp",
        "\"headers\": { \"Authorization\": \"Bearer {env:COMPANY_MCP_TOKEN}\" }",
        "\"timeout\": 5000",
    ] {
        assert!(text.contains(sample), "missing example: {sample}");
    }
    assert!(settings.mcp.is_empty(), "examples must stay commented out");
    assert!(settings.goal_sources.is_empty());
}

#[test]
fn native_mcp_examples_show_without_servers_and_preserve_existing_source_bindings() {
    let snapshot = json!({"config": {
        "mcp": {},
        "goal_sources": {"jira": {"server": "company", "tool": "get_issue"}}
    }});
    let text = settings_text(&snapshot);

    assert!(text.contains("your-mcp-server"));
    assert!(text.contains("\"tool\":\"get_issue\""));
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
fn native_mcp_discovery_request_is_bound_to_a_saved_server_and_revision() {
    let snapshot = json!({
        "revision": "rev-7",
        "config": {"mcp": {"company": {"type": "local", "command": ["company-mcp"]}}}
    });

    let request = discover_request(&snapshot, "company").unwrap();
    let request: Value = serde_json::from_slice(&request).unwrap();

    assert_eq!(request["kind"], "mcp_discover");
    assert_eq!(request["revision"], "rev-7");
    assert_eq!(request["server"], "company");
    assert!(discover_request(&snapshot, "unsaved-server").is_err());
}

#[test]
fn native_mcp_editor_exposes_discovery_for_registered_servers() {
    let snapshot = json!({
        "config": {"mcp": {"company": {"type": "local", "command": ["company-mcp"]}}}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut editor = State::default();
    editor.open(Some(&snapshot));
    let mut first = context.run_ui(Default::default(), |ui| {
        editor.show(Some(&snapshot), "en", true, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        editor.show(Some(&snapshot), "en", true, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    assert!(labels.contains(&"Discover tools"), "{labels:?}");
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
    let mut snapshot = worker
        .replies
        .recv_timeout(Duration::from_secs(15))
        .unwrap()
        .unwrap();
    worker
        .send(&save_request(&snapshot, &settings_text(&snapshot)).unwrap())
        .unwrap();
    assert_eq!(
        worker
            .replies
            .recv_timeout(Duration::from_secs(15))
            .unwrap()
            .unwrap()["saved"],
        true
    );
    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    snapshot = worker
        .replies
        .recv_timeout(Duration::from_secs(15))
        .unwrap()
        .unwrap();
    assert!(snapshot["config"]["mcp"].as_object().unwrap().is_empty());
    assert!(snapshot["job"].is_null());
    let text = r#"{"mcp":{"company":{"type":"local","command":["company-mcp"],"environment":{"TOKEN":"{env:COMPANY_TOKEN}"}}},"goal_sources":{}}"#;

    worker
        .send(&save_request(&snapshot, text).unwrap())
        .unwrap();
    let saved = worker
        .replies
        .recv_timeout(Duration::from_secs(15))
        .unwrap()
        .unwrap();

    assert_eq!(saved["saved"], true);
    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    let refreshed = worker
        .replies
        .recv_timeout(Duration::from_secs(15))
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
