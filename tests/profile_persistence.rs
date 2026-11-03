#![cfg(unix)]
//! R-48 / DD-56 / QAS-41: agent additions and removals are saved immediately and alone.
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
fn agent_list_is_saved_alone_and_survives_reopen() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("desktop");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&dir, exe).unwrap();
    let mut company = plan.agents[0].clone();
    company.id = "company".into();
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":[plan.agents[0].clone(), company.clone()],"repository":"","goal":"Saved goal","coordinator":"company","workspace_mode":"shared","roles":{"testing":["company","mock"],"development":["company"]}}})).unwrap();

    let mut added = plan.agents[0].clone();
    added.id = "company-2".into();
    mutate(&startup, serde_json::json!({"kind":"save_profiles","profiles":[plan.agents[0].clone(), company.clone(), added.clone()]})).unwrap();
    assert!(
        startup
            .dispatch(br#"{"kind":"save_profiles","revision":"stale","profiles":[]}"#)
            .is_err()
    );
    let mut invalid = added.clone();
    invalid.executable = "bash".into();
    assert!(
        mutate(
            &startup,
            serde_json::json!({"kind":"save_profiles","profiles":[invalid]})
        )
        .is_err()
    );
    drop(startup);

    let reopened = Startup::open(&dir, exe).unwrap();
    let config = snapshot(&reopened)["config"].clone();
    let ids: Vec<_> = config["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids, ["mock", "company", "company-2"]);
    assert_eq!(config["goal"], "Saved goal");

    // Removing the coordinator and a role agent keeps the settings consistent.
    mutate(
        &reopened,
        serde_json::json!({"kind":"save_profiles","profiles":[plan.agents[0].clone(), added]}),
    )
    .unwrap();
    let config = snapshot(&reopened)["config"].clone();
    assert_eq!(config["coordinator"], "mock");
    assert_eq!(
        config["roles"]["testing"],
        serde_json::json!([{"agent":"mock"}])
    );
    assert!(
        config["roles"].get("development").is_none(),
        "empty roles are removed"
    );
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}
