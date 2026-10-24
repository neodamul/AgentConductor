//! R-70 / DD-76 / QAS-63: English by default, Korean on demand, UTF-8 throughout. HOME is
//! set once in this test binary so the preference stays inside the fixture.
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
fn language_is_saved_embedded_and_validated() {
    let (root, _plan) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["language"],
        "en"
    );

    assert!(
        call(
            &room,
            serde_json::json!({"kind":"set_language","language":"xx"})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"set_language","language":"ko"}),
    )
    .unwrap();
    let stored =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored).unwrap()["language"],
        "ko"
    );
    drop(room);
    // A new window starts in the saved language.
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["language"],
        "ko"
    );
    call(
        &room,
        serde_json::json!({"kind":"set_language","language":"en"}),
    )
    .unwrap();
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
