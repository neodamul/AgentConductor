//! #899: the new workspace chooses from the ordered machine team, without launching it.
use agentconductor::ui::Room;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[path = "support/env.rs"]
mod test_env;
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn call(room: &Room, packet: Value) -> Value {
    room.dispatch(packet.to_string().as_bytes()).unwrap()
}
#[test]
fn new_workspace_defaults_in_list_order_and_preserves_saved_choices() {
    let root = Fixture(std::env::temp_dir().join(format!(
        "ac-899-{}",
        agentconductor::native_guard::session_id().unwrap()
    )));
    let home = root.0.join("home");
    let apps = home.join(".agentconductor");
    fs::create_dir_all(apps.join("desktop")).unwrap();
    let mut env = test_env::lock();
    env.set("HOME", &home);
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let profiles = json!([{"id":"z-first","parser":"custom","executable":exe,"args":[],"permission_mode":"unverified"},{"id":"a-second","parser":"custom","executable":exe,"args":[],"permission_mode":"unverified"}]);
    fs::write(
        apps.join("agents.json"),
        serde_json::to_vec(&profiles).unwrap(),
    )
    .unwrap();
    let first = apps.join("workspaces/first");
    let room = Room::startup(&first, exe).unwrap();
    let snapshot = call(&room, json!({"kind":"snapshot"}));
    assert_eq!(
        snapshot["config"]["coordinator"], "z-first",
        "new workspace uses first agent, not alphabetic order"
    );
    let stored: Value =
        serde_json::from_slice(&fs::read(first.join("startup.json")).unwrap()).unwrap();
    assert_eq!(
        stored["config"]["coordinator"], "z-first",
        "default is persisted before a user Save"
    );
    let mut config = snapshot["config"].clone();
    config["coordinator"] = json!("a-second");
    config["name"] = json!("First");
    call(
        &room,
        json!({"kind":"save","config":config,"revision":snapshot["revision"]}),
    );
    drop(room);
    let room = Room::startup(&first, exe).unwrap();
    assert_eq!(
        call(&room, json!({"kind":"snapshot"}))["config"]["coordinator"],
        "a-second",
        "reopen preserves a valid non-first choice"
    );
    let created = call(&room, json!({"kind":"create_workspace","name":"Second"}));
    assert_eq!(
        call(&room, json!({"kind":"snapshot"}))["config"]["coordinator"],
        "z-first",
        "menu creation uses the same default"
    );
    let path = Path::new(created["current"].as_str().unwrap());
    let stored: Value =
        serde_json::from_slice(&fs::read(path.join("startup.json")).unwrap()).unwrap();
    assert_eq!(stored["config"]["coordinator"], "z-first");
    drop(room);
    fs::write(apps.join("agents.json"), b"[]").unwrap();
    let empty = Room::startup(&apps.join("workspaces/empty"), exe).unwrap();
    let snapshot = call(&empty, json!({"kind":"snapshot"}));
    assert_eq!(snapshot["config"]["coordinator"], "");
    assert_eq!(snapshot["config"]["profiles"], json!([]));
    drop(empty);
}
