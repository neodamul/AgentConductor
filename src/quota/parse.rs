use super::{Snapshot, snapshot, window};
use serde_json::Value;

pub fn parse_codex(payload: &Value, observed_at: i64) -> Result<Snapshot, String> {
    let data = payload
        .get("result")
        .or_else(|| payload.get("params"))
        .unwrap_or(payload);
    let mut windows = Vec::new();
    let mut budgets = Vec::new();
    let mut add = |bucket: &str, value: &Value| -> Result<(), String> {
        if bucket.len() > 256 || !value.is_object() {
            return Err("invalid quota bucket".into());
        }
        budgets.extend(super::budget::parse(bucket, value)?);
        for label in ["primary", "secondary"] {
            if let Some(value) = value.get(label).filter(|v| !v.is_null()) {
                windows.push(window(bucket, label, value, false, None)?);
            }
        }
        Ok(())
    };
    if let Some(buckets) = data
        .get("rateLimitsByLimitId")
        .filter(|v| !v.is_null() && !v.as_object().is_some_and(|m| m.is_empty()))
    {
        let buckets = buckets.as_object().ok_or("invalid quota bucket map")?;
        if buckets.len() > 32 {
            return Err("too many quota buckets".into());
        }
        for (id, value) in buckets {
            add(id, value)?;
        }
    } else if let Some(value) = data.get("rateLimits").filter(|v| !v.is_null()) {
        add(
            value
                .get("limitId")
                .and_then(Value::as_str)
                .unwrap_or("codex"),
            value,
        )?;
    }
    let mut result = snapshot(
        "codex app-server account/rateLimits/read",
        observed_at,
        windows,
    )?;
    result.budgets = budgets;
    result.plan_type = data
        .get("rateLimits")
        .and_then(|v| v.get("planType"))
        .and_then(Value::as_str)
        .or_else(|| {
            data.get("rateLimitsByLimitId")
                .and_then(Value::as_object)
                .and_then(|b| {
                    b.values()
                        .find_map(|v| v.get("planType").and_then(Value::as_str))
                })
        })
        .filter(|v| v.len() <= 64)
        .map(str::to_owned);
    Ok(result)
}

pub fn parse_claude(payload: &Value, observed_at: i64) -> Result<Snapshot, String> {
    let mut windows = Vec::new();
    if let Some(limits) = payload.get("rate_limits").filter(|v| !v.is_null()) {
        if !limits.is_object() {
            return Err("invalid Claude quota windows".into());
        }
        for (label, minutes) in [
            ("five_hour", Some(300)),
            ("seven_day", Some(10080)),
            ("spend_limit", None),
        ] {
            if let Some(value) = limits.get(label).filter(|v| !v.is_null()) {
                windows.push(window("claude", label, value, true, minutes)?);
            }
        }
    }
    snapshot(
        "claude status-line rate_limits metadata",
        observed_at,
        windows,
    )
}
