use super::EventKind;
use serde_json::Value;

// DD-146: Codex emits recoverable transport notices as top-level error events.
// Only explicit retry/fallback announcements are nonterminal; final turn failures stay errors.
pub(super) fn codex_reconnecting(payload: &Value) -> bool {
    if payload["will_retry"] == false {
        return false;
    }
    let Some(message) = payload
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| payload.pointer("/error/message").and_then(Value::as_str))
    else {
        return false;
    };
    let message = message.trim();
    if let Some(attempt) = message.strip_prefix("Reconnecting... ") {
        let counter = attempt
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches(':');
        let Some((n, max)) = counter.split_once('/') else {
            return false;
        };
        return matches!((n.parse::<u32>(),max.parse::<u32>()),(Ok(n),Ok(max)) if n>0 && n<=max && max<=1000);
    }
    message.starts_with("Falling back from WebSockets to HTTPS transport")
        || message.starts_with("Falling back from WebSocket to HTTP transport")
}

pub(super) fn acio_kind(payload: &Value) -> EventKind {
    match payload["type"].as_str().unwrap_or("") {
        "session" => EventKind::Session,
        "progress" | "message" | "heartbeat" | "ack" => EventKind::Progress,
        "tool" => EventKind::Tool,
        "usage" => EventKind::Usage,
        "warning" => EventKind::Warning,
        "error" => EventKind::Error,
        "result"
            if matches!(
                payload["data"]["status"].as_str(),
                Some("failure" | "cancelled")
            ) =>
        {
            EventKind::Error
        }
        "result" => EventKind::Result,
        _ => EventKind::Unknown,
    }
}
