#![cfg(unix)]
//! GitHub #115: the workspace screen's chat request validates before launching any job.
use agentconductor::ui::Room;
use serde_json::json;

#[test]
fn chat_requests_are_validated_and_the_snapshot_carries_a_chat_slot() {
    let root = std::env::temp_dir().join(format!("ac-startup-chat-{}", std::process::id()));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(snapshot["chat"].is_null(), "no answer before any chat");
    let revision = snapshot["revision"].clone();
    let send = |turns: serde_json::Value| {
        room.dispatch(
            json!({"kind":"chat","revision":revision,"turns":turns})
                .to_string()
                .as_bytes(),
        )
    };
    assert!(send(json!([])).is_err(), "empty conversation");
    assert!(
        send(json!([{"role":"assistant","text":"hi"}])).is_err(),
        "must end with the operator"
    );
    assert!(
        !send(json!([{"role":"operator","text":"How do I run it?"}]))
            .unwrap_err()
            .is_empty(),
        "no coordinator is registered in a fresh workspace"
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
