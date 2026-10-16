#![cfg(feature = "test-fixtures")]
use agentconductor::{mcp::Settings, ui::setup::Startup};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn snapshot(startup: &Startup) -> Value {
    startup
        .dispatch(br#"{"kind":"snapshot","mcp_result":true}"#)
        .unwrap()
}
fn request(startup: &Startup, mut packet: Value) {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes()).unwrap();
}
fn settled(startup: &Startup) -> Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let state = snapshot(startup);
        if state["job"]["state"] != "running" {
            return state;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "MCP worker did not finish"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

#[tokio::test]
async fn registered_servers_need_document_bindings_with_actionable_guidance() {
    let settings =
        Settings::parse(r#"{"mcp":{"atlassian":{"type":"local","command":["never-called"]}}}"#)
            .unwrap();
    let preview =
        agentconductor::mcp::preview(&settings, "jira:EMM-1 confluence:123", Path::new("."))
            .await
            .unwrap();
    assert!(preview.records.is_empty());
    assert_eq!(preview.failures.len(), 2);
    for failure in preview.failures {
        let source = if failure.reference.kind == "jira" {
            "Jira"
        } else {
            "Confluence"
        };
        assert_eq!(
            failure.error,
            agentconductor::ui::messages::render("mcp.binding_missing", &[("source", source)])
        );
    }
}

#[test]
fn discover_then_coordinator_reads_without_bindings_and_reload() {
    let root = std::env::temp_dir().join(format!(
        "ac-source-binding-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let trace = root.join("calls.jsonl");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&root, exe).unwrap();
    let settings = Settings::parse(&json!({"mcp":{"atlassian":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","sources",trace]}}}).to_string()).unwrap();
    request(
        &startup,
        json!({"kind":"mcp_save","settings":serde_json::to_string(&settings).unwrap()}),
    );
    request(
        &startup,
        json!({"kind":"mcp_discover","server":"atlassian"}),
    );
    let catalog = settled(&startup);
    assert_eq!(catalog["job"]["state"], "completed", "{catalog}");
    assert_eq!(catalog["mcp_result"]["tools"].as_array().unwrap().len(), 2);
    assert!(!fs::read_to_string(&trace).unwrap().contains("tools/call"));
    let mut config = snapshot(&startup)["config"].clone();
    config["goal"] = "Implement jira:EMM-52599 using confluence:53553565".into();
    request(&startup, json!({"kind":"save","config":config}));
    let state = snapshot(&startup);
    assert!(
        startup
            .dispatch(
                json!({"kind":"mcp_preview","revision":state["revision"]})
                    .to_string()
                    .as_bytes()
            )
            .unwrap_err()
            .contains(agentconductor::ui::messages::text(
                "mcp.coordinator_required"
            ))
    );
    assert!(!fs::read_to_string(&trace).unwrap().contains("tools/call"));
    request(
        &startup,
        json!({"kind":"save_profiles","profiles":[{"id":"mcp-coordinator","executable":env!("CARGO_BIN_EXE_test_agent_fixture"),"args":["mcp-coordinator","normal"],"parser":"custom","input_mode":"stdin","permission_mode":"read_only"}]}),
    );
    request(&startup, json!({"kind":"mcp_preview"}));
    let state = settled(&startup);
    assert_eq!(state["job"]["state"], "completed", "{state}");
    let preview = &state["mcp_result"];
    assert!(
        preview["failures"].as_array().unwrap().is_empty(),
        "{preview}"
    );
    assert_eq!(preview["records"].as_array().unwrap().len(), 2);
    let calls = fs::read_to_string(&trace)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|v| v["method"] == "tools/call")
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    assert!(
        calls
            .iter()
            .any(|v| v["params"]["arguments"]["issue_key"] == "EMM-52599")
    );
    assert!(
        calls
            .iter()
            .any(|v| v["params"]["arguments"]["page_id"] == "53553565")
    );
    drop(startup);
    let reloaded = Startup::open(&root, exe).unwrap();
    let config = snapshot(&reloaded)["config"].clone();
    assert!(config["goal_sources"].as_object().unwrap().is_empty());
    assert_eq!(config["coordinator"], "mcp-coordinator");
    drop(reloaded);
    fs::remove_dir_all(root).unwrap();
}
