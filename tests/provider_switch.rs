#![cfg(unix)]
//! R-68 / DD-74 / QAS-61: a provider switch replaces the profile and renames references.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
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
