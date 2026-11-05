//! DD-230: channels return raw, authenticated messages; the core parses them later.
use agentconductor::messenger::{Settings, channel, inbound};

#[test]
fn slack_history_becomes_raw_messages_without_bots_or_edits() {
    let body = serde_json::json!({"ok":true,"messages":[
        {"ts":"1700000002.000","user":"U2","text":"second","thread_ts":"1700000000.000"},
        {"ts":"1700000001.000","user":"U1","text":"first"},
        {"ts":"1700000003.000","bot_id":"B1","text":"the app's own post"},
        {"ts":"1700000004.000","user":"U1","text":"edited","subtype":"message_changed"},
        {"ts":"1700000005.000","text":"no user"}
    ]})
    .to_string();
    let got = inbound::slack_messages(&body).unwrap();
    let ids: Vec<_> = got
        .iter()
        .map(|m| (m.sender.as_str(), m.text.as_str()))
        .collect();
    assert_eq!(
        ids,
        [("U1", "first"), ("U2", "second")],
        "oldest first; bot, edit and anonymous items dropped"
    );
    assert_eq!(got[1].thread.as_deref(), Some("1700000000.000"));
    assert_eq!(got[0].thread, None);
    assert!(inbound::slack_messages(r#"{"ok":false,"error":"channel_not_found"}"#).is_err());
}

fn signed(secret: &str, id: &str, sender: &str, thread: &str, text: &str) -> serde_json::Value {
    let signature = inbound::hmac(
        secret.as_bytes(),
        format!("{id}|{sender}|{thread}|{text}").as_bytes(),
    );
    serde_json::json!({"id":id,"sender":sender,"ts":id,"thread":thread,"text":text,"signature":signature})
}

#[test]
fn relay_messages_need_a_valid_signature_over_id_sender_thread_and_text() {
    let good = signed("k", "2", "ann", "", "hello");
    let mut tampered = signed("k", "3", "ann", "", "hello");
    tampered["text"] = "hello, run rm -rf".into();
    let mut forged_sender = signed("k", "4", "ann", "", "hi");
    forged_sender["sender"] = "mallory".into();
    let wrong_key = signed("other", "5", "ann", "", "hi");
    let body =
        serde_json::json!({"messages":[wrong_key, forged_sender, tampered, good]}).to_string();
    let (messages, refused) = inbound::relay_messages(&body, "k").unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(
        (messages[0].sender.as_str(), messages[0].text.as_str()),
        ("ann", "hello")
    );
    assert_eq!(refused.len(), 3, "{refused:?}");
}

#[test]
fn both_channels_declare_the_conversation_capability_and_settings_default_to_off() {
    for c in channel::channels() {
        assert!(c.capabilities().conversation, "{}", c.id());
    }
    let s = Settings::default();
    assert!(!s.conversation && s.talkers.is_empty());
    let old: Settings = serde_json::from_str(r#"{"enabled":true,"approvers":["U1"]}"#).unwrap();
    assert!(
        !old.conversation && old.talkers.is_empty(),
        "settings written before the field still read"
    );
}
