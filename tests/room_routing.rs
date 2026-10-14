//! User request 2026-10-14: the workspace page and the Agent Room stay open side by side.
//! Requests marked "room" go to the run; unmarked requests always reach the workspace.
use agentconductor::ui::Room;
use std::path::Path;

#[test]
fn marked_requests_go_to_the_room_and_others_to_the_workspace() {
    let root = std::env::temp_dir().join(format!(
        "ac-room-routing-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    unsafe { std::env::set_var("HOME", root.join("home")) };
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

#[test]
fn the_page_keeps_the_room_running_when_going_home() {
    let startup = include_str!("../src/ui/startup.js");
    let room = include_str!("../src/ui/room.js");
    assert!(room.contains("JSON.stringify({...packet,room:true})"));
    assert!(!startup.contains("request({kind:'home'});window.stopRoom"));
    assert!(
        startup.contains(
            "if(window.__acRoomRun!==result.run){window.stopRoom?.();window.startRoom();"
        )
    );
    assert!(startup.contains("q('back').classList?.toggle('run-live'"));
}
