use super::{
    Snapshot,
    parser::{snapshot, window},
};
use crate::{plan::AgentProfile, process::ProcessSpec};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

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

pub fn parse_custom_output(raw: &Value, pointer: Option<&str>) -> Result<Snapshot, String> {
    let value = match pointer {
        Some("") => raw,
        Some(pointer) if !pointer.starts_with('/') => {
            return Err("invalid quota output JSON Pointer".into());
        }
        Some(pointer) => raw
            .pointer(pointer)
            .ok_or("quota output JSON Pointer was not found")?,
        None => raw,
    };
    let snapshot: Snapshot = serde_json::from_value(value.clone())
        .map_err(|_| "custom quota output must match the normalized Snapshot JSON schema")?;
    if snapshot.source.is_empty()
        || snapshot.source.len() > 256
        || snapshot.observed_at <= 0
        || snapshot.windows.len() > 64
        || snapshot.budgets.len() > 64
        || snapshot.windows.iter().any(|window| {
            window.bucket.is_empty()
                || window.bucket.len() > 256
                || window.label.is_empty()
                || window.label.len() > 256
                || window
                    .used_percent
                    .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                || window
                    .remaining_percent
                    .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                || window.window_minutes == Some(0)
                || window.resets_at.is_some_and(|at| at <= 0)
        })
        || snapshot.budgets.iter().any(|budget| {
            budget.bucket.is_empty()
                || budget.bucket.len() > 256
                || budget.scope.is_empty()
                || budget.scope.len() > 256
                || budget
                    .remaining_percent
                    .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                || budget.resets_at.is_some_and(|at| at <= 0)
        })
    {
        return Err("custom quota output contains invalid or excessive values".into());
    }
    Ok(snapshot)
}

pub async fn query_custom(profile: &AgentProfile, dir: &Path) -> Result<Snapshot, String> {
    let config = profile
        .quota_command
        .as_ref()
        .ok_or("custom CLI quota command is not configured")?;
    if config.args.len() > 64
        || !(1..=300).contains(&config.timeout_seconds)
        || config
            .args
            .iter()
            .any(|arg| arg.len() > 4096 || arg.contains('\0'))
    {
        return Err("invalid custom quota command configuration".into());
    }
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|error| error.to_string())?;
    let dir = dir.canonicalize().map_err(|error| error.to_string())?;
    let executable = crate::executable::resolve(&profile.executable)
        .unwrap_or_else(|| profile.executable.clone().into());
    let args = config
        .args
        .iter()
        .map(|arg| {
            arg.replace("{workspace}", &dir.to_string_lossy())
                .replace("{agent_id}", &profile.id)
        })
        .collect();
    let result = crate::process::run(ProcessSpec {
        executable,
        args,
        cwd: dir.clone(),
        input: vec![],
        log_dir: dir.join("process"),
        timeout: Duration::from_secs(u64::from(config.timeout_seconds)),
        signal_grace: Duration::from_secs(3),
        environment: profile.environment.clone(),
    })
    .await?;
    if result.timed_out {
        return Err("custom quota command timed out; inspect retained process logs".into());
    }
    if result.exit_code != Some(0) {
        return Err(format!(
            "custom quota command exited with {}; inspect retained process logs",
            result
                .exit_code
                .map_or_else(|| "no code".into(), |code| code.to_string())
        ));
    }
    let output = tokio::fs::read(&result.stdout)
        .await
        .map_err(|error| error.to_string())?;
    if output.len() > 1024 * 1024 {
        return Err("custom quota output exceeds 1 MiB".into());
    }
    let raw: Value = serde_json::from_slice(&output)
        .map_err(|_| "custom quota command did not return one JSON object")?;
    parse_custom_output(&raw, config.output_pointer.as_deref())
}
