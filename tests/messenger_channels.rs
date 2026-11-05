//! #515 phase 1 / DD-228: channels sit behind one registry; stored settings stay compatible.
use agentconductor::messenger::{self, Kind, Settings, channel};

#[test]
fn the_registry_lists_slack_and_webhook_with_their_capabilities() {
    let ids: Vec<_> = channel::channels().iter().map(|c| c.id()).collect();
    assert_eq!(ids, ["slack", "webhook"]);
    for c in channel::channels() {
        let caps = c.capabilities();
        assert!(
            caps.notice && caps.replies,
            "{} sends notices and reads decision replies",
            c.id()
        );
        assert!(!caps.conversation, "{} has no conversation yet", c.id());
    }
    assert!(channel::find("slack").is_some() && channel::find("teams").is_none());
    assert!(!Kind::Slack.channel().uses_template() && Kind::Webhook.channel().uses_template());
}

#[test]
fn kind_is_stored_as_the_same_text_and_unknown_ids_are_refused() {
    assert_eq!(serde_json::to_string(&Kind::Slack).unwrap(), "\"slack\"");
    assert_eq!(
        serde_json::to_string(&Kind::Webhook).unwrap(),
        "\"webhook\""
    );
    assert_eq!(
        serde_json::from_str::<Kind>("\"webhook\"").unwrap(),
        Kind::Webhook
    );
    let error = serde_json::from_str::<Kind>("\"teams\"")
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown messenger channel"), "{error}");
}

#[test]
fn settings_written_before_the_interface_still_read_and_write_the_same_kind() {
    let old: Settings = serde_json::from_str(r#"{"enabled":true,"kind":"webhook"}"#).unwrap();
    assert_eq!(old.kind, Kind::Webhook);
    let default: Settings = serde_json::from_str("{}").unwrap();
    assert_eq!(
        default.kind,
        Kind::Slack,
        "a missing kind is Slack, as before"
    );
    let json = serde_json::to_value(&old).unwrap();
    assert_eq!(json["kind"], "webhook");
    let _ = messenger::EVENTS;
}
