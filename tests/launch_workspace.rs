#![cfg(unix)]
//! GitHub #226: the app opens the last used saved workspace; the default workspace is created only when none is saved.
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
fn a_deleted_default_workspace_does_not_come_back_and_the_last_used_one_opens() {
    let root = std::env::temp_dir().join(format!("ac-launch-{}", std::process::id()));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let mut env = env::lock();
    env.set("HOME", &home);
    let data = home.join(".agentconductor");
    let desktop = data.join("desktop");
    assert_eq!(
        workspaces::launch_root().unwrap(),
        desktop,
        "with nothing saved the default is the one to create"
    );
    let room = Room::startup(&desktop, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    save_named(&room, "First");
    let other = call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Second"}),
    )
    .unwrap();
    drop(other);
    let listed = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let names: Vec<_> = listed["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.contains(&"First".to_string()) && names.contains(&"Second".to_string()),
        "{names:?}"
    );
    // "Second" is open now: it is the last used one.
    assert_eq!(
        workspaces::launch_root().unwrap().file_name(),
        listed["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .find(|w| w["name"] == "Second")
            .map(|w| std::path::PathBuf::from(w["path"].as_str().unwrap())
                .file_name()
                .unwrap()
                .to_owned())
            .as_deref()
    );
    // Delete the default workspace while "Second" is open, then restart.
    let default = listed["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["name"] == "First")
        .unwrap()
        .clone();
    call(
        &room,
        serde_json::json!({"kind":"delete_workspace","path":default["path"],"name":"First"}),
    )
    .unwrap();
    drop(room);
    assert!(!desktop.join("startup.json").exists());
    let next = workspaces::launch_root().unwrap();
    assert_ne!(next, desktop, "the deleted default is not reopened");
    assert!(next.join("startup.json").is_file());
    let room = Room::startup(&next, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let names: Vec<_> =
        call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap()["workspaces"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w["name"].as_str().unwrap().to_string())
            .collect();
    assert_eq!(names, ["Second"], "only what the operator keeps is listed");
    assert!(!desktop.exists(), "no default folder is recreated");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
