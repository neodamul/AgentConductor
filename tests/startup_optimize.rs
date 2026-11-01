#![cfg(unix)]
//! GitHub #180: the workspace screen's optimize request validates before launching any job.
use agentconductor::ui::Room;
use serde_json::json;

#[test]
fn optimize_requests_are_validated_and_the_snapshot_carries_an_optimize_slot() {
    let root = std::env::temp_dir().join(format!("ac-startup-optimize-{}", std::process::id()));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(
        snapshot["optimize"].is_null(),
        "no proposal before any request"
    );
    let revision = snapshot["revision"].clone();
    let send = |goal: &str| {
        room.dispatch(
            json!({"kind":"optimize_goal","revision":revision,"goal":goal})
                .to_string()
                .as_bytes(),
        )
    };
    assert!(send("   ").is_err(), "empty goal");
    assert!(send(&"a".repeat(9000)).is_err(), "goal over 8 KiB");
    let error = send("Make it faster").unwrap_err();
    assert!(
        error.to_lowercase().contains("coordinator") || error.contains("코디네이터"),
        "valid optimization requests must reach coordinator validation: {error}"
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
