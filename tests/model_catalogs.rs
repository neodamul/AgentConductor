#![cfg(unix)]
//! R-49 / DD-57 / QAS-42: model lists are prepared automatically and survive saves.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = include_str!("../src/ui/startup.js");

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn page_discovers_missing_catalogs_one_at_a_time_without_touching_plans_or_edits() {
    let auto = &SCRIPT[SCRIPT
        .find("function autoDiscover(")
        .expect("auto discovery")..];
    let auto = &auto[..auto.find("\n").unwrap()];
    for condition in [
        "busy()",
        "state.proposal",
        "dirty",
        "'custom'",
        "catalog",
        "kind:'discover'",
        "addedAgents",
    ] {
        assert!(auto.contains(condition), "missing {condition}: {auto}");
    }
}

#[test]
fn loading_models_keeps_a_reviewed_plan_approvable() {
    let (root, plan) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":root.join("repo"),"goal":"g","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    mutate(&startup, serde_json::json!({"kind":"draft"})).unwrap();
    mutate(
        &startup,
        serde_json::json!({"kind":"edit","tasks":plan.tasks,"workspace_mode":"shared"}),
    )
    .unwrap();
    let catalog = serde_json::json!({"models":["fixture-small"],"source":"fixture","queried_at":1,"cli_version":"1"});
    mutate(
        &startup,
        serde_json::json!({"kind":"catalog","agent":"mock","catalog":catalog}),
    )
    .unwrap();
    let state = snapshot(&startup);
    assert_eq!(
        state["proposal"]["phase"], "ready",
        "observed models do not discard the plan"
    );
    mutate(
        &startup,
        serde_json::json!({"kind":"start","digest":state["proposal"]["digest"]}),
    )
    .expect("plan stays approvable");
    // Closing the UI leaves execution alive; await the owned job before fixture cleanup.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while snapshot(&startup)["job"]["state"] == "running" {
        assert!(
            std::time::Instant::now() < deadline,
            "owned fixture job did not finish"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn saving_without_a_catalog_keeps_the_observed_one() {
    let (root, plan) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let catalog = serde_json::json!({"models":["fixture-small"],"source":"fixture","queried_at":1,"cli_version":"1"});
    let mut with = serde_json::to_value(&plan.agents[0]).unwrap();
    with["catalog"] = catalog.clone();
    let config = |profile: &serde_json::Value| serde_json::json!({"profiles":[profile],"repository":"","goal":"g","coordinator":"mock","workspace_mode":"shared"});
    mutate(
        &startup,
        serde_json::json!({"kind":"save","config":config(&with)}),
    )
    .unwrap();
    let without = serde_json::to_value(&plan.agents[0]).unwrap();
    mutate(
        &startup,
        serde_json::json!({"kind":"save","config":config(&without)}),
    )
    .unwrap();
    assert_eq!(
        snapshot(&startup)["config"]["profiles"][0]["catalog"],
        catalog,
        "save keeps catalog"
    );
    mutate(
        &startup,
        serde_json::json!({"kind":"save_profiles","profiles":[without.clone()]}),
    )
    .unwrap();
    assert_eq!(
        snapshot(&startup)["config"]["profiles"][0]["catalog"],
        catalog,
        "save_profiles keeps catalog"
    );
    let mut changed = without;
    changed["executable"] = "/usr/bin/true".into();
    mutate(
        &startup,
        serde_json::json!({"kind":"save_profiles","profiles":[changed]}),
    )
    .unwrap();
    assert!(
        snapshot(&startup)["config"]["profiles"][0]["catalog"].is_null(),
        "a different executable drops it"
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn model_updates_are_quiet_and_only_for_new_agents() {
    let auto = &SCRIPT[SCRIPT.find("function autoDiscover(").unwrap()..];
    let auto = &auto[..auto.find("\n").unwrap()];
    assert!(
        !auto.contains("notice("),
        "automatic discovery stays silent"
    );
    let profiles = &SCRIPT[SCRIPT.find("function profiles(").unwrap()..];
    let stamp = &profiles[..profiles.find("if(stamp===profileStamp)").unwrap()];
    assert!(
        !stamp.contains("draft.roles"),
        "role edits must not rebuild the table"
    );
    let refresh = &SCRIPT[SCRIPT
        .find("function refreshRoleModels(")
        .expect("in-place refresh")..];
    let refresh = &refresh[..refresh.find("\n").unwrap()];
    assert!(
        refresh.contains("document.activeElement"),
        "focused selector is left alone"
    );
}

#[test]
fn saving_unrelated_settings_keeps_the_plan() {
    let (root, plan) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let config = |name: &str, goal: &str| serde_json::json!({"profiles":plan.agents,"repository":root.join("repo"),"goal":goal,"coordinator":"mock","workspace_mode":"shared","name":name,"roles":{"testing":["mock"]}});
    mutate(
        &startup,
        serde_json::json!({"kind":"save","config":config("A","g")}),
    )
    .unwrap();
    mutate(&startup, serde_json::json!({"kind":"draft"})).unwrap();
    mutate(
        &startup,
        serde_json::json!({"kind":"save","config":config("Renamed","g")}),
    )
    .unwrap();
    assert_eq!(
        snapshot(&startup)["proposal"]["phase"],
        "ready",
        "a rename keeps the plan"
    );
    mutate(
        &startup,
        serde_json::json!({"kind":"save","config":config("Renamed","new goal")}),
    )
    .unwrap();
    assert!(
        snapshot(&startup)["proposal"].is_null(),
        "a new goal clears it"
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
