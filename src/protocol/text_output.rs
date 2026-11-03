use super::{Event, EventKind};
use serde_json::json;

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

fn reads_as_refusal(text: &str) -> bool {
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

pub(super) fn event(seq: u64, raw_offset: u64, session_id: Option<String>, line: &[u8]) -> Event {
    let text = excerpt(line);
    let (kind, payload) = if reads_as_refusal(&text) {
        (
            EventKind::Error,
            json!({"type":"error","message":format!(
                "check the CLI configuration: the CLI printed what reads as a refusal: {text}"
            )}),
        )
    } else {
        (EventKind::Progress, json!({"type":"text","text":text}))
    };
    Event {
        seq,
        raw_offset,
        session_id,
        kind,
        payload,
    }
}
