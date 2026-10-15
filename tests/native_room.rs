#![cfg(unix)]
use agentconductor::{ui::Room, workflow};
#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn native_requests_are_typed_bounded_and_hold_a_single_window_lease() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::open(&dir, exe).unwrap();
    assert!(Room::open(&dir, exe).is_err());
    for packet in [
        b"{".as_slice(),
        br#"{"kind":"snapshot","extra":true}"#,
        br#"{"kind":"shell","command":"rm"}"#,
        br#"{"kind":"action","revision":"invalid","action":{"kind":"resume","extra":true}}"#,
    ] {
        assert!(room.dispatch(packet).is_err());
    }
    for operation in ["snapshot", "evidence"] {
        let packet = serde_json::json!({"kind":operation,"extra":true});
        assert!(
            room.dispatch(packet.to_string().as_bytes())
                .unwrap_err()
                .contains("unknown field")
        );
    }
    for operation in ["resume", "recover"] {
        let packet = serde_json::json!({"kind":"action","revision":"invalid","action":{"kind":operation,"extra":true}});
        assert!(
            room.dispatch(packet.to_string().as_bytes())
                .unwrap_err()
                .contains("unknown field")
        );
    }
    assert!(room.dispatch(&vec![b' '; 65537]).is_err());
    let state = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(state["report"]["state"], "waiting_approval");
    assert!(state["quota_refresh_due"].as_object().unwrap().is_empty());
    let stale = serde_json::json!({"kind":"action","revision":"0".repeat(64),"action":{"kind":"approve","sha":state["report"]["candidate"]["commit"]}});
    assert!(room.dispatch(stale.to_string().as_bytes()).is_err());
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    drop(room);
    drop(Room::open(&dir, exe).unwrap());
    std::fs::remove_dir_all(root).unwrap();
}
