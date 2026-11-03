#![cfg(unix)]
//! R-101 / DD-109 (verified for GitHub #349): the agent CLI list is the machine's master list.
//! Another app-owned workspace opens with it, and deleting a workspace leaves it in place.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

fn call(room: &Room, value: serde_json::Value) -> serde_json::Value {
    room.dispatch(value.to_string().as_bytes()).unwrap()
}

fn agent_ids(snapshot: &serde_json::Value) -> Vec<String> {
    snapshot["config"]["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn every_workspace_opens_with_the_master_list_and_deleting_one_keeps_it() {
    let (root, plan) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let apps = root.join("home/.agentconductor");
    let (first, second) = (
        apps.join("workspaces/first"),
        apps.join("workspaces/second"),
    );
    std::fs::create_dir_all(apps.join("desktop")).unwrap();
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));

    let room = Room::startup(&first, exe).unwrap();
    let revision = call(&room, serde_json::json!({"kind":"snapshot"}))["revision"].clone();
    let config = serde_json::json!({"profiles":plan.agents,"repository":root.join("repo"),
        "goal":"ladder","coordinator":"mock","workspace_mode":"shared","name":"First"});
    call(
        &room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    );
    let saved = agent_ids(&call(&room, serde_json::json!({"kind":"snapshot"})));
    assert!(!saved.is_empty());
    drop(room);

    let other = Room::startup(&second, exe).unwrap();
    assert_eq!(
        agent_ids(&call(&other, serde_json::json!({"kind":"snapshot"}))),
        saved,
        "a new workspace opens with the master list"
    );
    drop(other);

    let room = Room::startup(&first, exe).unwrap();
    let deleted = room.dispatch(
        serde_json::json!({"kind":"delete_workspace","path":first.canonicalize().unwrap(),"name":"First"})
            .to_string()
            .as_bytes(),
    );
    assert!(deleted.is_ok(), "{deleted:?}");
    drop(room);
    assert!(
        apps.join("agents.json").exists(),
        "deleting a workspace keeps the master list"
    );
    let reopened = Room::startup(&second, exe).unwrap();
    assert_eq!(
        agent_ids(&call(&reopened, serde_json::json!({"kind":"snapshot"}))),
        saved
    );
    drop(reopened);
    drop(env);
    let _ = std::fs::remove_dir_all(root);
}
