#![cfg(unix)]
//! R-40 / DD-48 / QAS-32: named workspaces and an explicit registry. HOME is set once in
//! this test binary so the registry stays inside the fixture.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
fn save(
    room: &Room,
    name: &str,
    profiles: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let revision = call(room, serde_json::json!({"kind":"snapshot"}))?["revision"].clone();
    let config = serde_json::json!({"profiles":profiles,"repository":"","goal":"","coordinator":"mock","workspace_mode":"shared","name":name});
    call(
        room,
        serde_json::json!({"kind":"save","revision":revision,"config":config}),
    )
}

#[test]
fn name_field_sits_above_the_goal() {
    let name = HTML
        .find("id=\"start-name\"")
        .expect("workspace name field");
    assert!(name < HTML.find("id=\"start-goal\"").unwrap());
}

#[test]
fn workspaces_are_named_created_listed_and_removed_without_deleting_files() {
    let (root, plan) = support::setup("shared");
    let home = root.join("home");
    unsafe { std::env::set_var("HOME", &home) };
    let profiles = serde_json::to_value(&plan.agents).unwrap();
    let first = root.join("first");
    let room = Room::startup(&first, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    save(&room, "Payments team", &profiles).unwrap();
    assert!(save(&room, &"x".repeat(81), &profiles).is_err());
    assert!(save(&room, "bad\nname", &profiles).is_err());
    let snapshot = call(&room, serde_json::json!({"kind":"snapshot"})).unwrap();
    assert_eq!(snapshot["config"]["name"], "Payments team");

    let created = call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Client A"}),
    )
    .unwrap();
    let path = created["current"].as_str().unwrap().to_string();
    assert!(
        path.starts_with(
            home.canonicalize()
                .unwrap()
                .join(".agentconductor/workspaces/client-a")
                .to_str()
                .unwrap()
        )
    );
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["config"]["name"],
        "Client A"
    );
    let again = call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Client A"}),
    )
    .unwrap();
    assert_ne!(
        again["current"], created["current"],
        "a second workspace gets its own folder"
    );
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"create_workspace","name":""})
        )
        .is_err()
    );

    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    let names: Vec<_> = listed["registered"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap().to_string())
        .collect();
    assert!(
        names.contains(&"Payments team".to_string())
            && names.iter().filter(|n| *n == "Client A").count() == 2,
        "{names:?}"
    );

    let current = listed["current"].clone();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"forget_workspace","path":current})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"forget_workspace","path":path}),
    )
    .unwrap();
    let listed = call(&room, serde_json::json!({"kind":"workspaces"})).unwrap();
    assert!(
        !listed["registered"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w["path"] == path.as_str())
    );
    assert!(
        Path::new(&path).join("startup.json").exists(),
        "removing from the list keeps files"
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn menu_offers_new_and_open_existing_workspace() {
    let script = include_str!("../src/ui/startup.js");
    assert!(script.contains("'New workspace…'"));
    assert!(script.contains("'Open existing workspace…'"));
    assert!(!script.contains("Open existing folder"));
}
