//! Native theme selection persists as a validated workspace preference.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

#[test]
fn theme_is_saved_embedded_and_validated() {
    let (root, _plan) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let call = |room: &Room, v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"set_theme","theme":"sepia"})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"set_theme","theme":"dark"}),
    )
    .unwrap();
    drop(room);
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    let prefs =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert!(prefs.contains("\"theme\": \"dark\""), "{prefs}");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
