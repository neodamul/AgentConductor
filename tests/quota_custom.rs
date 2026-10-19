use agentconductor::{plan::AgentProfile, quota};
use serde_json::json;

#[test]
fn custom_quota_output_maps_a_configured_json_pointer_to_snapshot() {
    let raw = json!({
        "account": {
            "source":"aipro quota --json",
            "observed_at":1_800_000_000_i64,
            "windows":[{"bucket":"team","label":"weekly","used_percent":27.0,"remaining_percent":73.0,"window_minutes":10080,"resets_at":1_800_100_000_i64}],
            "budgets":[],"plan_type":"enterprise"
        }
    });
    let snapshot = quota::parse_custom_output(&raw, Some("/account")).unwrap();
    assert_eq!(snapshot.plan_type.as_deref(), Some("enterprise"));
    assert_eq!(
        snapshot.view(1_800_000_001).windows[0].remaining_percent,
        Some(73.0)
    );
}

#[test]
fn custom_quota_output_rejects_invalid_percent_and_bad_pointer() {
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"internal","parser":"custom","executable":"aipro","args":[],
        "quota_command":{"args":["quota","--json"],"output_pointer":"/quota"}
    }))
    .unwrap();
    let mut raw = json!({"quota":{"source":"aipro","observed_at":1_800_000_000_i64,"windows":[{"bucket":"team","label":"week","used_percent":140.0}],"budgets":[]}});
    assert!(profile.quota_command.is_some());
    assert!(quota::parse_custom_output(&raw, Some("not/a/pointer")).is_err());
    assert!(quota::parse_custom_output(&raw, Some("/quota")).is_err());
    raw["quota"]["windows"][0]["used_percent"] = json!(20.0);
    assert!(quota::parse_custom_output(&raw, Some("/quota")).is_ok());
}

#[cfg(unix)]
#[tokio::test]
async fn custom_quota_command_runs_argv_and_retains_process_logs() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let raw = json!({"quota":{"source":"fixture","observed_at":now,"windows":[],"budgets":[]}});
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"internal","parser":"custom","executable":"/bin/echo","args":[],
        "quota_command":{"args":[raw.to_string()],"output_pointer":"/quota"}
    }))
    .unwrap();
    let dir = std::env::temp_dir().join(format!("ac-custom-quota-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let snapshot = quota::query_custom(&profile, &dir).await.unwrap();
    assert_eq!(snapshot.source, "fixture");
    assert!(dir.join("process/stdout.jsonl").is_file());
    assert!(dir.join("process/stderr.log").is_file());
    std::fs::remove_dir_all(dir).unwrap();
}
