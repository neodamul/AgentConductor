//! DD-133: the default workspace (~/.agentconductor/desktop) can be deleted while it is open.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}

#[test]
fn the_open_default_workspace_is_deleted_and_an_empty_one_opens() {
    let (root, plan) = support::setup("shared");
    let home = root.join("home");
    let mut env = test_env::lock();
    env.set("HOME", &home);
    let desktop = home.join(".agentconductor/desktop");
    std::fs::create_dir_all(&desktop).unwrap();
    let room = Room::startup(&desktop, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let revision = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["revision"].clone();
    let config = serde_json::json!({"profiles":plan.agents,"repository":root.join("repo"),"goal":"ladder","coordinator":"mock","workspace_mode":"shared","name":"Ladder game"});
    call(
        &room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    )
    .unwrap();
    let listed = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let entry = listed["workspaces"][0].clone();
    assert_eq!(entry["name"], "Ladder game");
    call(
        &room,
        serde_json::json!({"kind":"delete_workspace","path":entry["path"],"name":entry["name"]}),
    )
    .unwrap();
    let now = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(now["config"]["goal"], "");
    assert!(!desktop.join("startup.json").exists());
    assert!(root.join("repo/base.txt").exists(), "project files stay");
    drop(room);
    let _ = std::fs::remove_dir_all(root);
}
