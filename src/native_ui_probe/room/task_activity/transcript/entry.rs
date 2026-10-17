use super::{Entry, Tone, format};
use serde_json::Value;

pub(super) fn unknown(entries: &mut Vec<Entry>, kind: &str, raw: &Value) {
    let message = raw
        .pointer("/payload/summary")
        .or_else(|| raw.get("summary"))
        .or_else(|| raw.get("text"))
        .or_else(|| raw.get("message"));
    if kind.is_empty() && message.is_none() {
        return;
    }
    let text = message
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    add(
        entries,
        if kind.is_empty() { "room.status" } else { kind },
        &text,
        Tone::Muted,
    );
}

pub(super) fn add(entries: &mut Vec<Entry>, label: &str, text: &str, tone: Tone) {
    if !text.is_empty() {
        entries.push(Entry {
            label: label.into(),
            text: text.into(),
            tone,
        });
    }
}

pub(super) fn add_value(entries: &mut Vec<Entry>, label: &str, value: &Value, tone: Tone) {
    add(
        entries,
        label,
        &format::clip(&format::result_text(value), 4000),
        tone,
    );
}
