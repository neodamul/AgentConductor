use super::{Snapshot, snapshot, window};
use serde_json::{Value, json};

/// Normalize the official Claude CLI wire format, without inferring missing usage.
pub fn parse_claude_event(payload: &Value, observed_at: i64) -> Result<Snapshot, String> {
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
    if profile.parser != crate::protocol::Provider::Claude {
        return Ok(());
    }
    for event in events
        .iter()
        .filter(|event| event.payload["type"] == "rate_limit_event")
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        match parse_claude_event(&event.payload, now) {
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
