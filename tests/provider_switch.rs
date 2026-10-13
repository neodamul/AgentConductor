#![cfg(unix)]
//! R-68 / DD-74 / QAS-61: a provider switch replaces the profile and renames references.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn permission_follows_the_cli_and_is_not_chosen() {
    // User decision 2026-10-13: the profile dialog has no permission-level menu.
    assert!(!HTML.contains("id=\"start-permission\""));
    assert!(!SCRIPT.contains("q('permission')"));
}

#[test]
fn provider_comes_before_agent_id() {
    let dialog = &HTML[HTML.find("<dialog id=\"start-agent-dialog\"").unwrap()..];
    let provider = dialog.find("id=\"start-provider\"").unwrap();
    let id = dialog.find("id=\"start-agent-id\"").unwrap();
    assert!(provider < id, "Provider is the first field");
}

#[test]
fn provider_change_replaces_every_template_field() {
    let change = &SCRIPT[SCRIPT.find("q('provider').onchange=").unwrap()..];
    let change = &change[..change.find('\n').unwrap()];
    for key in [
        "q('agent-id').value=uniqueId(",
        "resolve_executable",
        "q('sessions').value",
        "q('quota-minutes').value",
        "q('args').value",
        "q('input').value",
    ] {
        assert!(change.contains(key), "provider change misses {key}");
    }
    // The previous CLI's models never follow the agent to another provider.
    assert!(SCRIPT.contains("delete p.catalog"));
    assert!(
        SCRIPT.contains("renames"),
        "a changed ID is sent as a rename"
    );
}

#[test]
fn unique_id_appends_a_suffix() {
    let start = SCRIPT.find("function uniqueId(").unwrap();
    let body = &SCRIPT[start..];
    let body = &body[..body.find('\n').unwrap()];
    let script = format!(
        "{body}\nconst a=require('node:assert/strict');a.equal(uniqueId('codex',[]),'codex');a.equal(uniqueId('codex',['codex']),'codex-2');a.equal(uniqueId('codex',['codex','codex-2']),'codex-3');"
    );
    let status = std::process::Command::new("node")
        .args(["-e", &script])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn renamed_agent_keeps_coordinator_roles_and_fallbacks() {
    let (root, plan) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let mut company = plan.agents[0].clone();
    company.id = "company".into();
    company.quota_group = Some("company-account".into());
    let mut helper = plan.agents[0].clone();
    helper.id = "helper".into();
    helper.fallback = Some(agentconductor::plan::Fallback {
        agent: "company".into(),
        model: None,
    });
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":[company.clone(), helper.clone()],"repository":"","goal":"","coordinator":"company","workspace_mode":"shared","roles":{"testing":["company","helper"],"development":[{"agent":"company"}]}}})).unwrap();

    let mut renamed = company.clone();
    renamed.id = "renamed".into();
    let saved = mutate(&startup, serde_json::json!({"kind":"save_profiles","profiles":[renamed, helper],"renames":{"company":"renamed"}})).unwrap();
    let config = &saved["config"];
    assert_eq!(config["coordinator"], "renamed");
    assert_eq!(config["profiles"][1]["fallback"]["agent"], "renamed");
    let roles = config["roles"].to_string();
    assert!(!roles.contains("company"), "{roles}");
    assert_eq!(config["roles"]["development"][0]["agent"], "renamed");
    assert_eq!(config["roles"]["testing"].as_array().unwrap().len(), 2);

    // Without a rename, a removed ID drops its references, fallbacks included.
    let only = config["profiles"][1].clone();
    let saved = mutate(
        &startup,
        serde_json::json!({"kind":"save_profiles","profiles":[only]}),
    )
    .unwrap();
    assert_eq!(saved["config"]["coordinator"], "helper");
    assert!(saved["config"]["profiles"][0]["fallback"].is_null());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
