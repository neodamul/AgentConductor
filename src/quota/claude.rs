use super::{Snapshot, snapshot, window};
use serde_json::{Value, json};

/// Normalize the official Claude CLI wire format, without inferring missing usage.
pub fn parse_claude_event(payload: &Value, observed_at: i64) -> Result<Snapshot, String> {
    if payload.get("api_error").is_some_and(|v| !v.is_null())
        || payload
            .get("api_error_status")
            .is_some_and(|v| v.as_i64().is_some_and(|n| n >= 400))
    {
        let r = crate::availability::detect(payload, &payload.to_string(), observed_at)
            .ok_or("not a Claude availability restriction")?;
        let mut s = snapshot("claude stream-json api_error", observed_at, vec![])?;
        s.restriction = Some(r);
        return Ok(s);
    }
    if payload["type"] != "rate_limit_event" {
        return Err("not a Claude rate limit event".into());
    }
    let info = payload["rate_limit_info"]
        .as_object()
        .ok_or("invalid Claude quota event")?;
    if !matches!(
        info.get("status").and_then(Value::as_str),
        Some("allowed" | "allowed_warning" | "rejected")
    ) {
        return Err("invalid Claude quota status".into());
    }
    let mut windows = Vec::new();
    if let Some(label) = info.get("rateLimitType").filter(|v| !v.is_null()) {
        let label = label.as_str().ok_or("invalid Claude quota window type")?;
        let minutes = match label {
            "five_hour" => Some(300),
            "seven_day" | "seven_day_opus" | "seven_day_sonnet" => Some(10080),
            "overage" => None,
            _ => return Err("unknown Claude quota window type".into()),
        };
        let used = match info.get("utilization") {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                value
                    .as_f64()
                    .filter(|n| n.is_finite() && (0.0..=1.0).contains(n))
                    .ok_or("invalid Claude quota utilization")?
                    * 100.0,
            ),
        };
        windows.push(window(
            "claude",
            label,
            &json!({"used_percentage":used,"resets_at":info.get("resetsAt")}),
            true,
            minutes,
        )?);
    }
    snapshot("claude stream-json rate_limit_event", observed_at, windows)
}

/// Only owned stdout events reach this observer. Malformed metadata is diagnostic.
pub fn observe_events(
    store: &mut crate::state::Store,
    profile: &crate::plan::AgentProfile,
    events: &[crate::protocol::Event],
) -> Result<(), String> {
    if !matches!(
        profile.parser,
        crate::protocol::Provider::Claude | crate::protocol::Provider::Custom
    ) {
        return Ok(());
    }
    for event in events.iter().filter(|event| {
        if profile.parser == crate::protocol::Provider::Custom {
            event.kind == crate::protocol::EventKind::Usage
                && profile
                    .quota_rules
                    .iter()
                    .any(|rule| event.payload["type"].as_str() == Some(rule.event_type.as_str()))
        } else {
            event.payload["type"] == "rate_limit_event"
                || event.payload.get("api_error").is_some_and(|v| !v.is_null())
                || event.payload["api_error_status"]
                    .as_i64()
                    .is_some_and(|n| n >= 400)
        }
    }) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let parsed = if profile.parser == crate::protocol::Provider::Custom {
            super::custom::parse_custom_event(profile, &event.payload, now)
        } else {
            parse_claude_event(&event.payload, now)
        };
        match parsed {
            Ok(snapshot) => super::persist(store, profile, Ok(snapshot))?,
            Err(error) => store.record(
                "quota-warning",
                &json!({"agent":profile.id,"seq":event.seq,"error":error}),
                "Ignored malformed Claude quota metadata; task stream retained",
            )?,
        }
    }
    Ok(())
}
