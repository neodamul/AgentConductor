use super::{Snapshot, snapshot, window};
use crate::plan::AgentProfile;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomQuotaRule {
    pub event_type: String,
    pub bucket_pointer: String,
    pub label_pointer: String,
    pub used_percent_pointer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resets_at_pointer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_minutes_pointer: Option<String>,
}

fn valid_pointer(pointer: &str) -> bool {
    pointer.len() <= 256
        && (pointer.is_empty()
            || pointer.strip_prefix('/').is_some_and(|parts| {
                parts.split('/').all(|part| {
                    let mut chars = part.chars();
                    while let Some(ch) = chars.next() {
                        if ch == '~' && !matches!(chars.next(), Some('0' | '1')) {
                            return false;
                        }
                    }
                    true
                })
            }))
}

pub fn validate_custom_rules(rules: &[CustomQuotaRule]) -> Result<(), String> {
    if rules.len() > 16 {
        return Err("too many custom quota rules".into());
    }
    let mut events = std::collections::BTreeSet::new();
    for rule in rules {
        if rule.event_type.is_empty()
            || rule.event_type.len() > 128
            || !events.insert(&rule.event_type)
        {
            return Err("custom quota event types must be unique and bounded".into());
        }
        let pointers = [
            Some(rule.bucket_pointer.as_str()),
            Some(rule.label_pointer.as_str()),
            Some(rule.used_percent_pointer.as_str()),
            rule.resets_at_pointer.as_deref(),
            rule.window_minutes_pointer.as_deref(),
        ];
        if pointers.into_iter().flatten().any(|p| !valid_pointer(p)) {
            return Err("custom quota rules require valid JSON Pointers".into());
        }
    }
    Ok(())
}

fn text<'a>(event: &'a Value, pointer: &str, field: &str) -> Result<&'a str, String> {
    event
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 256)
        .ok_or_else(|| format!("invalid custom quota {field}"))
}

fn optional(event: &Value, pointer: Option<&str>) -> Value {
    pointer
        .and_then(|pointer| event.pointer(pointer))
        .cloned()
        .unwrap_or(Value::Null)
}

pub fn parse_custom_event(
    profile: &AgentProfile,
    event: &Value,
    observed_at: i64,
) -> Result<Snapshot, String> {
    if profile.parser != crate::protocol::Provider::Custom {
        return Err("custom quota rules require a custom CLI profile".into());
    }
    let event_type = event
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let rule = profile
        .quota_rules
        .iter()
        .find(|rule| rule.event_type == event_type)
        .ok_or("no custom quota rule matches this event")?;
    let bucket = text(event, &rule.bucket_pointer, "bucket")?;
    let label = text(event, &rule.label_pointer, "window label")?;
    let used = event
        .pointer(&rule.used_percent_pointer)
        .cloned()
        .unwrap_or(Value::Null);
    if !used
        .as_f64()
        .is_some_and(|value| value.is_finite() && (0.0..=100.0).contains(&value))
    {
        return Err("invalid custom quota used percentage".into());
    }
    let normalized = json!({
        "usedPercent": used,
        "resetsAt": optional(event, rule.resets_at_pointer.as_deref()),
        "windowDurationMins": optional(event, rule.window_minutes_pointer.as_deref())
    });
    let quota_window = window(bucket, label, &normalized, false, None)?;
    snapshot(
        "custom CLI JSONL quota event",
        observed_at,
        vec![quota_window],
    )
}
