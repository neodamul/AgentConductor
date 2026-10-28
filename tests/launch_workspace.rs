#![cfg(unix)]
//! GitHub #226: nothing is listed or opened as a "default" workspace. The app opens the last used saved one;
//! with none, it asks for a name, and the unused placeholder is never shown and is removed once a workspace exists.
use agentconductor::ui::{Room, workspaces};
use std::path::{Path, PathBuf};
#[path = "support/env.rs"]
mod env;

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
fn names(room: &Room) -> Vec<String> {
    let saved = call(room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let menu = call(room, serde_json::json!({"kind":"workspaces"})).unwrap();
    let from_menu: Vec<_> = menu["registered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap().to_string())
        .collect();
    let from_saved: Vec<_> = saved["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(from_menu, from_saved, "the menu and the saved list agree");
    from_saved
}
fn open(root: &Path) -> Room {
    Room::startup(root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap()
}

#[test]
fn a_blank_placeholder_is_never_listed_and_a_deleted_workspace_does_not_come_back() {
    let root = std::env::temp_dir().join(format!("ac-launch-{}", std::process::id()));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let mut env = env::lock();
    env.set("HOME", &home);
    let desktop = home.join(".agentconductor/desktop");

    // First run: nothing is saved, so the placeholder opens, is flagged blank and is not listed.
    assert_eq!(workspaces::launch_root().unwrap(), desktop);
    let room = open(&desktop);
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["workspace_blank"],
        true
    );
    assert!(names(&room).is_empty(), "no default workspace is shown");

    // Naming the first workspace replaces the placeholder.
    call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"First"}),
    )
    .unwrap();
    assert!(!desktop.exists(), "the unused placeholder is removed");
    assert_eq!(names(&room), ["First"]);
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["workspace_blank"],
        false
    );

    // A second workspace; delete the first; restart: the second opens and nothing else is back.
    call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Second"}),
    )
    .unwrap();
    let listed = call(&room, serde_json::json!({"kind":"saved_workspaces"})).unwrap();
    let first = listed["workspaces"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["name"] == "First")
        .unwrap()
        .clone();
    call(
        &room,
        serde_json::json!({"kind":"delete_workspace","path":first["path"],"name":"First"}),
    )
    .unwrap();
    drop(room);
    let next: PathBuf = workspaces::launch_root().unwrap();
    assert!(next.join("startup.json").is_file());
    let room = open(&next);
    assert_eq!(names(&room), ["Second"]);
    assert!(!desktop.exists(), "no default folder is recreated");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_default_folder_with_real_work_stays_listed_and_opens() {
    let root = std::env::temp_dir().join(format!("ac-launch-used-{}", std::process::id()));
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let mut env = env::lock();
    env.set("HOME", &home);
    let desktop = home.join(".agentconductor/desktop");
    let room = open(&desktop);
    let revision = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["revision"].clone();
    let config = serde_json::json!({"profiles":[],"repository":"","goal":"ship it","coordinator":"","workspace_mode":"shared","name":"Mine"});
    call(
        &room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    )
    .unwrap();
    assert_eq!(
        names(&room),
        ["Mine"],
        "a named workspace is real even in the default folder"
    );
    drop(room);
    assert_eq!(
        workspaces::launch_root().unwrap().canonicalize().unwrap(),
        desktop.canonicalize().unwrap()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_page_asks_for_a_first_workspace_name_that_cannot_be_skipped() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/first_workspace_runtime.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
