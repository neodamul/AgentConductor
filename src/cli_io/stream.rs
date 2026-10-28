//! Rules over a whole captured stream that JSON Schema cannot express on one line.
use super::{timestamp, validate};
use serde_json::Value;

/// Extra rules that depend on how the run was started.
#[derive(Default, Clone, Copy)]
pub struct Rules {
    /// The input role was `coordinator`: a successful `result` must carry `data` (docs/cli-io-protocol.md §14).
    pub coordinator: bool,
}

pub fn validate_stream(kind: &str, text: &str, rules: Rules) -> Result<(), String> {
    let (line_kind, events) = match kind {
        "events" => ("event", true),
        "controls" => ("control", false),
        _ => {
            return Err(format!(
                "unknown stream kind {kind}; use events or controls"
            ));
        }
    };
    let mut expected_seq = 1;
    let mut last_ts: Option<i64> = None;
    let mut first: Option<(String, bool)> = None;
    let mut any_session_id = false;
    let mut result_seen = false;
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let at = index + 1;
        let doc: Value =
            serde_json::from_str(line).map_err(|e| format!("line {at}: not JSON: {e}"))?;
        validate(line_kind, &doc).map_err(|errors| format!("line {at}: {}", errors[0]))?;
        let seq = doc["seq"].as_u64().unwrap_or(0);
        if seq != expected_seq {
            return Err(format!("line {at}: seq {seq} should be {expected_seq}"));
        }
        expected_seq += 1;
        if !events {
            continue;
        }
        let ts = timestamp::parse(doc["ts"].as_str().unwrap_or("")).unwrap_or(0);
        if last_ts.is_some_and(|last| ts < last) {
            return Err(format!("line {at}: ts goes backwards"));
        }
        last_ts = Some(ts);
        if result_seen {
            return Err(format!("line {at}: event after the result"));
        }
        let kind = doc["type"].as_str().unwrap_or("").to_string();
        let has_session = doc.get("session_id").is_some();
        any_session_id |= has_session;
        first.get_or_insert((kind.clone(), has_session));
        if kind == "result" {
            result_seen = true;
            if rules.coordinator
                && doc["data"]["status"] == "success"
                && !doc["data"]["data"].is_object()
            {
                return Err(format!(
                    "line {at}: a coordinator's successful result needs data"
                ));
            }
        }
    }
    if events {
        if any_session_id && first.as_ref().is_some_and(|(kind, _)| kind != "session") {
            return Err(
                "the first event must be a session event when events carry a session_id".into(),
            );
        }
        if !result_seen {
            return Err("the stream has no result event".into());
        }
    } else if expected_seq == 1 {
        return Err("the stream has no control lines".into());
    }
    Ok(())
}
