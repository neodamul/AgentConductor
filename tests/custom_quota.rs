use agentconductor::{plan::AgentProfile, quota::custom::parse_custom_event};
use serde_json::json;

fn profile() -> AgentProfile {
    serde_json::from_value(json!({
        "id":"aipro", "executable":"aipro", "args":[], "parser":"custom",
        "quota_rules":[{
            "event_type":"usage",
            "bucket_pointer":"/payload/account",
            "label_pointer":"/payload/period",
            "used_percent_pointer":"/payload/used",
            "resets_at_pointer":"/payload/reset",
            "window_minutes_pointer":"/payload/minutes"
        }]
    }))
    .unwrap()
}

#[test]
fn declarative_custom_usage_maps_to_normalized_quota_window() {
    let snapshot = parse_custom_event(
        &profile(),
        &json!({"type":"usage","payload":{"account":"team","period":"weekly","used":73.0,"reset":1_800_000_000,"minutes":10080}}),
        1_700_000_000,
    )
    .unwrap();
    let window = &snapshot.windows[0];
    assert_eq!(window.bucket, "team");
    assert_eq!(window.label, "weekly");
    assert_eq!(window.used_percent, Some(73.0));
    assert_eq!(window.remaining_percent, Some(27.0));
    assert_eq!(window.resets_at, Some(1_800_000_000));
    assert_eq!(window.window_minutes, Some(10080));
}

#[test]
fn malformed_and_duplicate_rules_are_rejected() {
    let mut value = serde_json::to_value(profile()).unwrap();
    value["quota_rules"][0]["used_percent_pointer"] = "not-a-pointer".into();
    assert!(serde_json::from_value::<AgentProfile>(value).is_err());

    let mut value = serde_json::to_value(profile()).unwrap();
    let duplicate = value["quota_rules"][0].clone();
    value["quota_rules"].as_array_mut().unwrap().push(duplicate);
    assert!(serde_json::from_value::<AgentProfile>(value).is_err());
}

#[test]
fn quota_rule_count_boundary_is_sixteen() {
    let mut value = serde_json::to_value(profile()).unwrap();
    let template = value["quota_rules"][0].clone();
    let rules = value["quota_rules"].as_array_mut().unwrap();
    for index in 1..16 {
        let mut rule = template.clone();
        rule["event_type"] = format!("usage-{index}").into();
        rules.push(rule);
    }
    assert!(serde_json::from_value::<AgentProfile>(value.clone()).is_ok());
    value["quota_rules"].as_array_mut().unwrap().push(template);
    assert!(serde_json::from_value::<AgentProfile>(value).is_err());
}

#[test]
fn out_of_range_observations_fail_without_partial_quota() {
    let error = parse_custom_event(
        &profile(),
        &json!({"type":"usage","payload":{"account":"team","period":"weekly","used":101}}),
        1_700_000_000,
    )
    .unwrap_err();
    assert!(error.contains("used percentage"));
}
