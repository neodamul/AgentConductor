//! Line classifiers: recoverable Codex notices, `acio.event/1` kinds, excerpts and refusals.
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

/// The normalized kind of one `acio.event/1` line; a failed or cancelled result is an error.
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

/// A bounded, single-line quote of raw output for a failure message.
pub(super) fn excerpt(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let line: String = text.trim().chars().take(200).collect();
    let line = line.replace(['\n', '\r'], " ");
    if text.trim().chars().count() > 200 {
        format!("{line}…")
    } else {
        line
    }
}

/// Words a CLI uses when it refuses to run a tool; text like this is a configuration problem, not progress.
pub(super) fn reads_as_refusal(text: &str) -> bool {
    let text = text.to_lowercase();
    [
        "permission denied",
        "denied by",
        "not permitted",
        "not allowed",
        "approval required",
        "requires approval",
    ]
    .iter()
    .any(|phrase| text.contains(phrase))
}
