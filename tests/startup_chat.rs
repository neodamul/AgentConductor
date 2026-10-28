#![cfg(unix)]
//! GitHub #115: the workspace screen's chat request validates before launching any job.
use agentconductor::ui::Room;
use serde_json::json;
#[path = "support/env.rs"]
mod env;

#[test]
fn chat_requests_are_validated_and_the_snapshot_carries_a_chat_slot() {
    let root = std::env::temp_dir().join(format!("ac-startup-chat-{}", std::process::id()));
    // GitHub #222: the request diagnostics log lives under HOME; a test must not write to the real one.
    let mut env = env::lock();
    env.set("HOME", root.join("home"));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(snapshot["chat"].is_null(), "no answer before any chat");
    let revision = snapshot["revision"].clone();
    let send = |text: &str| {
        room.dispatch(
            json!({"kind":"chat","revision":revision,"text":text})
                .to_string()
                .as_bytes(),
        )
    };
    assert!(send("  ").is_err(), "empty question");
    assert!(send(&"x".repeat(9 * 1024)).is_err(), "over-long question");
    assert!(
        !send("How do I run it?").unwrap_err().is_empty(),
        "no coordinator is registered in a fresh workspace"
    );
    // GitHub #185: the conversation is stored with the workspace and can be cleared.
    let history = room.dispatch(br#"{"kind":"chat_history"}"#).unwrap();
    assert_eq!(
        history["turns"],
        json!([]),
        "nothing stored before a chat ran"
    );
    let reset = room.dispatch(br#"{"kind":"chat_reset"}"#).unwrap();
    assert_eq!(reset["turns"], json!([]));
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
