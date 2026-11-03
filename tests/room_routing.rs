//! User request 2026-10-14: the workspace page and the Agent Room stay open side by side.
//! Requests marked "room" go to the run; unmarked requests always reach the workspace.
use agentconductor::ui::Room;
use std::path::Path;

#[path = "support/env.rs"]
mod test_env;

#[test]
fn marked_requests_go_to_the_room_and_others_to_the_workspace() {
    let root = std::env::temp_dir().join(format!(
        "ac-room-routing-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let room = Room::startup(
        &root.join("workspace"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let workspace = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(workspace["config"].is_object(), "workspace snapshot");
    // No run is open yet: a room request is refused instead of reaching the workspace.
    assert!(
        room.dispatch(br#"{"kind":"snapshot","room":true}"#)
            .is_err()
    );
    // Going home never fails, even with nothing open.
    room.dispatch(br#"{"kind":"home"}"#).unwrap();
    drop(room);
    let _ = std::fs::remove_dir_all(root);
}
