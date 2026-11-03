#![cfg(feature = "test-fixtures")]
use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
fn snapshot(s: &Startup) -> Value {
    s.dispatch(br#"{"kind":"snapshot","mcp_result":true}"#)
        .unwrap()
}
#[test]
fn mcp_settings_are_portable_referenced_and_saved_without_changing_goal() {
    let root = std::env::temp_dir().join(format!(
        "ac-mcp-ui-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let s = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = snapshot(&s);
    let settings = json!({"mcp":{"company":{"type":"local","command":["company-mcp"],"environment":{"TOKEN":"{env:COMPANY_TOKEN}"}}},"goal_sources":{}});
    s.dispatch(
        json!({"kind":"mcp_save","revision":state["revision"],"settings":settings.to_string()})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let state = snapshot(&s);
    assert_eq!(
        state["config"]["mcp"]["company"]["environment"]["TOKEN"],
        "{env:COMPANY_TOKEN}"
    );
    assert_eq!(state["config"]["goal"], "");
    assert!(s.dispatch(json!({"kind":"mcp_apply","revision":state["revision"],"preview_id":"fake","selected":[]}).to_string().as_bytes()).is_err());
    drop(s);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn worker_preview_apply_and_portable_export_keep_private_bodies_local() {
    let root = std::env::temp_dir().join(format!(
        "ac-mcp-worker-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let s = Startup::open(&root, exe).unwrap();
    let settings = json!({"mcp":{"company":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server"]}},"goal_sources":{"jira":{"server":"company","tool":"get_issue","arguments":{"id":"{id}"},"read_only":true}}});
    let state = snapshot(&s);
    s.dispatch(
        json!({"kind":"mcp_save","revision":state["revision"],"settings":settings.to_string()})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let state = snapshot(&s);
    let mut config = state["config"].clone();
    config["goal"] = "Implement jira:EMM-52599".into();
    s.dispatch(
        json!({"kind":"save","revision":state["revision"],"config":config})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let state = snapshot(&s);
    assert!(
        s.dispatch(
            json!({"kind":"prepare","revision":state["revision"]})
                .to_string()
                .as_bytes()
        )
        .unwrap_err()
        .contains(agentconductor::ui::messages::text(
            "native.preview_and_apply_referenced_jira_confluence_sources_before"
        ))
    );
    s.dispatch(json!({"kind":"save_profiles","revision":state["revision"],"profiles":[{"id":"mcp-coordinator","executable":env!("CARGO_BIN_EXE_test_agent_fixture"),"args":["mcp-coordinator","normal"],"parser":"custom","input_mode":"stdin","permission_mode":"read_only"}]}).to_string().as_bytes()).unwrap();
    let state = snapshot(&s);
    s.dispatch(
        json!({"kind":"mcp_preview","revision":state["revision"]})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let state = loop {
        let state = snapshot(&s);
        if state["job"]["state"] != "running" {
            break state;
        }
        assert!(std::time::Instant::now() < end);
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    assert_eq!(state["job"]["state"], "completed");
    assert_eq!(
        state["mcp_result"]["records"][0]["reference"]["id"],
        "EMM-52599"
    );
    s.dispatch(json!({"kind":"mcp_apply","revision":state["revision"],"preview_id":state["mcp_result"]["id"],"selected":["jira:EMM-52599"]}).to_string().as_bytes()).unwrap();
    let state = snapshot(&s);
    assert_eq!(state["imports"]["selected"][0], "jira:EMM-52599");
    let exported = root.join("export");
    std::fs::create_dir(&exported).unwrap();
    s.dispatch(
        json!({"kind":"export_settings","revision":state["revision"],"folder":exported})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let file = std::fs::read_to_string(exported.join("agentconductor.workspace.json")).unwrap();
    assert!(file.contains("get_issue"));
    assert!(!file.contains("Replay is rejected"));
    assert!(!file.contains("imports"));
    let other = root.join("other");
    std::fs::create_dir(&other).unwrap();
    let imported = Startup::open(&other, exe).unwrap();
    let v = snapshot(&imported);
    imported.dispatch(json!({"kind":"import_settings","revision":v["revision"],"folder":exported,"keep_verified":false}).to_string().as_bytes()).unwrap();
    assert_eq!(
        snapshot(&imported)["config"]["mcp"]["company"]["type"],
        "local"
    );
    drop(imported);
    let mut config = state["config"].clone();
    config["goal"] = "Implement jira:EMM-2".into();
    s.dispatch(
        json!({"kind":"save","revision":state["revision"],"config":config})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    assert!(snapshot(&s)["imports"].is_null());
    drop(s);
    std::fs::remove_dir_all(root).unwrap();
}
