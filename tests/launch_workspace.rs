//! Main #226: reopen the last saved workspace; create the default only for a first run.
use agentconductor::ui::{Room, workspaces};
use std::path::Path;

#[path = "support/env.rs"]
mod env;

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}

fn save_named(room: &Room, name: &str) {
    let revision = call(room, serde_json::json!({"kind":"snapshot"})).unwrap()["revision"].clone();
    let config = serde_json::json!({"profiles":[],"repository":"","goal":"","coordinator":"","workspace_mode":"shared","name":name});
    call(
        room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    )
    .unwrap();
}

#[test]
fn deleted_default_stays_deleted_and_last_saved_workspace_reopens() {
    let root = std::env::temp_dir().join(format!("ac-launch-{}", std::process::id()));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let mut env = env::lock();
    env.set("HOME", &home);
    let desktop = home.join(".agentconductor/desktop");

    assert_eq!(workspaces::launch_root().unwrap(), desktop);
    let room = Room::startup(&desktop, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    save_named(&room, "First");
    call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Second"}),
    )
    .unwrap();

    let listed = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let second = listed["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|workspace| workspace["name"] == "Second")
        .unwrap()["path"]
        .as_str()
        .unwrap();
    assert_eq!(
        workspaces::launch_root().unwrap(),
        Path::new(second).canonicalize().unwrap()
    );
    let first = listed["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|workspace| workspace["name"] == "First")
        .unwrap();
    call(
        &room,
        serde_json::json!({"kind":"delete_workspace","path":first["path"],"name":"First"}),
    )
    .unwrap();
    drop(room);

    let launch = workspaces::launch_root().unwrap();
    assert_ne!(launch, desktop);
    assert!(launch.join("startup.json").is_file());
    let room = Room::startup(&launch, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let reopened = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let names: Vec<_> = reopened["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|workspace| workspace["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Second"]);
    assert!(!desktop.exists());
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
