use agentconductor::quota::{due_by_profile, refresh_due};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn per_agent_refresh_policy_matches_the_configured_interval() {
    assert!(refresh_due(None, 0, 5));
    assert!(!refresh_due(Some(0), 299_999, 5));
    assert!(refresh_due(Some(0), 300_000, 5));
    assert!(refresh_due(Some(10), 0, 5));
    assert!(refresh_due(Some(0), 60_000, 1));
    assert!(!refresh_due(Some(0), 300_000, 10));
    assert!(refresh_due(Some(0), 600_000, 10));
    assert!(!refresh_due(Some(0), 60_000, 0));
}

#[test]
fn due_map_uses_each_paid_profile_interval_and_omits_custom_agents() {
    let profiles: Vec<agentconductor::plan::AgentProfile> = serde_json::from_value(json!([
        {"id":"claude","parser":"claude","executable":"claude","args":[],"quota_refresh_minutes":1},
        {"id":"codex","parser":"codex","executable":"codex","args":[],"quota_refresh_minutes":5},
        {"id":"internal","parser":"custom","executable":"aipro","args":[],"quota_refresh_minutes":1}
    ]))
    .unwrap();
    let last = BTreeMap::from([("claude".into(), 0), ("codex".into(), 0)]);
    let due = due_by_profile(&profiles, &last, 60_000);
    assert_eq!(due["claude"], true);
    assert_eq!(due["codex"], false);
    assert!(due.get("internal").is_none());
}
