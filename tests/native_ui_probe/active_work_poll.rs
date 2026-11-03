use crate::native_ui_probe::{Pending, Probe};
use serde_json::json;
use std::time::{Duration, Instant};

#[test]
fn reopening_an_active_chat_does_not_schedule_a_fast_snapshot_loop() {
    let mut probe = Probe::default();
    super::super::workspace_ops::load_workspace(
        &mut probe,
        &json!({
            "chat_history": [{"role":"operator","text":"Resume"}],
            "job": {"kind":"chat","state":"running","id":"chat-job"}
        }),
    );

    assert!(probe.coordinator_chat.awaiting());
    assert!(probe.workspace_next_refresh.is_none());
}

#[test]
fn active_chat_recovery_does_not_replace_the_thirty_second_fallback_with_fast_polling() {
    let mut probe = Probe::default();
    probe.coordinator_chat.started("chat-job".into());
    probe.workspace_next_refresh = Some(Instant::now() + Duration::from_millis(500));

    probe.chat_refreshed(json!({
        "job": {"kind":"chat","state":"running","id":"chat-job"}
    }));

    assert!(probe.workspace_next_refresh.is_none());
}

#[test]
fn active_goal_optimizer_recovery_does_not_schedule_fast_polling() {
    let mut probe = Probe::default();
    probe.optimize_started(json!({"accepted":true,"id":"optimizer-job"}));
    probe.workspace_next_refresh = Some(Instant::now() + Duration::from_millis(500));

    probe.optimize_refreshed(json!({
        "job": {"kind":"optimize","state":"running","id":"optimizer-job"}
    }));

    assert!(probe.workspace_next_refresh.is_none());
}

#[test]
fn active_mcp_lookup_does_not_schedule_fast_polling() {
    let mut probe = Probe {
        pending: Some(Pending::McpStatus),
        ..Probe::default()
    };

    super::super::mcp::apply(&mut probe, json!({"job":{"kind":"mcp","state":"running"}}));

    assert!(probe.workspace_next_refresh.is_none());
}
