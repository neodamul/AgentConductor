use agentconductor::{
    plan::AgentProfile,
    protocol::{Decoder, EventKind},
    quota,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn custom_profile_configures_event_mapping_and_session_pointer() {
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"internal","parser":"custom","executable":"aipro","args":[],
        "event_types":{"finished":"result","working":"progress","oops":"error"},
        "session_id_pointer":"/meta/session"
    }))
    .unwrap();
    let mut decoder = Decoder::for_profile(&profile, 1024);
    let events = decoder
        .push(b"{\"type\":\"working\",\"meta\":{\"session\":\"s-1\"}}\n{\"type\":\"finished\"}\n")
        .unwrap();
    assert_eq!(events[0].kind, EventKind::Progress);
    assert_eq!(events[0].session_id.as_deref(), Some("s-1"));
    assert_eq!(events[1].kind, EventKind::Result);
}

#[test]
fn legacy_custom_profile_keeps_existing_event_defaults() {
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"internal","parser":"custom","executable":"aipro","args":[]
    }))
    .unwrap();
    let mut decoder = Decoder::for_profile(&profile, 1024);
    assert_eq!(
        decoder.push(b"{\"type\":\"result\"}\n").unwrap()[0].kind,
        EventKind::Result
    );
    assert_eq!(
        decoder.push(b"{\"type\":\"other\"}\n").unwrap()[0].kind,
        EventKind::Unknown
    );
}

#[test]
fn custom_quota_pointer_normalizes_and_validates_snapshot() {
    let raw = json!({"account":{"source":"aipro quota","observed_at":1_800_000_000_i64,
        "windows":[{"bucket":"team","label":"weekly","used_percent":27.0,
            "remaining_percent":73.0,"window_minutes":10080,"resets_at":1_800_100_000_i64}],
        "budgets":[],"plan_type":"enterprise"}});
    let snapshot = quota::parse_custom_output(&raw, Some("/account")).unwrap();
    assert_eq!(snapshot.plan_type.as_deref(), Some("enterprise"));
    assert_eq!(
        snapshot.view(1_800_000_001).windows[0].remaining_percent,
        Some(73.0)
    );
    assert!(quota::parse_custom_output(&raw, Some("/missing")).is_err());
}

#[test]
fn custom_quota_refresh_is_enabled_only_when_configured() {
    let configured: AgentProfile = serde_json::from_value(json!({
        "id":"internal","parser":"custom","executable":"aipro","args":[],
        "quota_command":{"args":["quota","--json"]}
    }))
    .unwrap();
    let unconfigured: AgentProfile = serde_json::from_value(json!({
        "id":"other","parser":"custom","executable":"aipro","args":[]
    }))
    .unwrap();
    let due = agentconductor::quota::due_by_profile(
        &[configured, unconfigured],
        &BTreeMap::new(),
        60_000,
    );
    assert_eq!(due["internal"], true);
    assert!(due.get("other").is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn custom_quota_runs_registered_executable_without_a_shell_and_keeps_logs() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let payload =
        json!({"quota":{"source":"agent={agent_id}","observed_at":now,"windows":[],"budgets":[]}})
            .to_string();
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"internal","parser":"custom","executable":"/bin/echo","args":[],
        "quota_command":{"args":[payload],"output_pointer":"/quota"}
    }))
    .unwrap();
    let dir = std::env::temp_dir().join(format!(
        "ac-custom-quota-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let snapshot = quota::query(&profile, &dir).await.unwrap();
    assert_eq!(snapshot.source, "agent=internal");
    assert!(dir.join("process/stdout.jsonl").is_file());
    assert!(dir.join("process/stderr.log").is_file());
    std::fs::remove_dir_all(dir).unwrap();
}
