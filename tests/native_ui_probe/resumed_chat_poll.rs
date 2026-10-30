use super::super::super::Probe;
use serde_json::json;

#[test]
fn reopening_with_a_running_chat_schedules_a_status_refresh() {
    let mut probe = Probe::default();
    super::load_workspace(
        &mut probe,
        &json!({
            "chat_history": [{"role":"operator", "text":"Keep going"}],
            "job": {"kind":"chat", "state":"running", "id":"chat-job"}
        }),
    );

    assert!(probe.coordinator_chat.awaiting());
    assert!(probe.workspace_next_refresh.is_some());
}

#[test]
fn running_chat_refreshes_until_completion_reenables_the_editor() {
    let mut probe = Probe::default();
    probe.coordinator_chat.started("chat-job".into());
    probe.chat_refreshed(json!({
        "job": {"kind":"chat", "state":"running", "id":"chat-job"}
    }));

    assert!(probe.coordinator_chat.awaiting());
    assert!(probe.workspace_next_refresh.is_some());

    probe.chat_refreshed(json!({
        "job": {"kind":"chat", "state":"completed", "id":"chat-job"},
        "chat": {"answer":"The answer survived the UI restart."}
    }));

    assert!(!probe.coordinator_chat.awaiting());
    assert!(probe.workspace_next_refresh.is_none());
}
