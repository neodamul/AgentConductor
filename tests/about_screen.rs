//! GitHub #468: the About screen appears on the first start after an install or an update and stays reachable from
//! the menu. The last version it was shown for is an app preference; nothing is polled and nothing is sent anywhere.
use agentconductor::ui::Room;
use std::path::Path;

#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

#[test]
fn about_is_shown_once_per_installed_version() {
    let (root, _plan) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let call = |room: &Room, v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    let version = agentconductor::update::current();

    let fresh = call(&room, serde_json::json!({"kind":"about_status"})).unwrap();
    assert_eq!(fresh["show"], true, "a fresh install shows it: {fresh}");
    assert_eq!(fresh["version"], version.as_str());
    assert_eq!(
        fresh["updated"], false,
        "nothing was shown before, so it is not an update"
    );

    call(&room, serde_json::json!({"kind":"about_seen"})).unwrap();
    let seen = call(&room, serde_json::json!({"kind":"about_status"})).unwrap();
    assert_eq!(
        seen["show"], false,
        "the same version is not shown again: {seen}"
    );
    let saved =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert!(saved.contains("about_seen_version"), "{saved}");

    // An update: the stored version differs from the running one.
    let path = root.join("home/.agentconductor/preferences.json");
    let mut preferences: serde_json::Value = serde_json::from_str(&saved).unwrap();
    preferences["about_seen_version"] = "0.0.1".into();
    std::fs::write(&path, preferences.to_string()).unwrap();
    let updated = call(&room, serde_json::json!({"kind":"about_status"})).unwrap();
    assert_eq!(updated["show"], true);
    assert_eq!(
        updated["updated"], true,
        "a stored older version marks an update: {updated}"
    );

    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
