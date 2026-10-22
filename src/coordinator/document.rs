use super::*;
use crate::process;
use crate::protocol::{Decoder, EventKind};
use std::{fs, io::Read, path::Path};

/// The whole reply as JSON, or else the first fenced ```json block that carries `field`.
/// Models often wrap the requested object in a Markdown fence despite "return only JSON".
fn json_reply(text: &str, field: &str) -> Option<Value> {
    let has = |v: &Value| v.get(field).is_some();
    if let Ok(json) = serde_json::from_str::<Value>(text.trim()) {
        return Some(json).filter(has);
    }
    let mut rest = text;
    while let Some(open) = rest.find("```") {
        let after = &rest[open + 3..];
        let body_start = after.find('\n')? + 1;
        let body = &after[body_start..];
        let close = body.find("```")?;
        if let Ok(json) = serde_json::from_str::<Value>(body[..close].trim())
            && has(&json)
        {
            return Some(json);
        }
        rest = &body[close + 3..];
    }
    None
}

/// The last `limit` bytes of a log, as text (empty when unreadable).
fn log_tail(path: &Path, limit: u64) -> String {
    use std::io::{Seek, SeekFrom};
    let Ok(mut file) = fs::File::open(path) else {
        return String::new();
    };
    let len = file.metadata().map_or(0, |m| m.len());
    let _ = file.seek(SeekFrom::Start(len.saturating_sub(limit)));
    let mut bytes = vec![];
    let _ = file.take(limit).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes).trim().to_string()
}

/// The text of an error event, bounded, for the operator.
fn event_error(payload: &Value) -> String {
    let text = [
        "/api_error",
        "/error/message",
        "/message",
        "/error",
        "/result",
        "/item/message",
    ]
    .iter()
    .find_map(|p| payload.pointer(p).and_then(Value::as_str))
    .map(str::to_owned)
    .unwrap_or_else(|| payload.to_string());
    let text = if let Some(status) = payload["api_error_status"].as_i64() {
        format!("{text} (HTTP {status})")
    } else {
        text
    };
    text.chars().take(2000).collect()
}

/// Why a coordinator process failed: its status and the end of its own output, so the
/// operator sees the cause instead of only "logs retained".
pub fn failure(label: &str, result: &process::ProcessResult) -> String {
    let status = if result.stopped {
        "stopped".to_string()
    } else if result.timed_out {
        "timed out".to_string()
    } else {
        format!(
            "exit code {}",
            result.exit_code.map_or("none".into(), |c| c.to_string())
        )
    };
    let stderr = log_tail(&result.stderr, 1500);
    let stdout = log_tail(&result.stdout, 1500);
    let detail = if !stderr.is_empty() {
        stderr
    } else {
        // Without stderr, the last output line (often a JSON error event) carries the cause.
        stdout.lines().last().unwrap_or_default().to_string()
    };
    let logs = result.stdout.parent().unwrap_or(Path::new("")).display();
    if detail.is_empty() {
        format!("{label} failed ({status}); logs: {logs}")
    } else {
        format!("{label} failed ({status}): {detail}\nlogs: {logs}")
    }
}

pub fn read_document(provider: Provider, stdout: &Path, field: &str) -> Result<Value, String> {
    read_with_decoder(Decoder::new(provider, 1024 * 1024), provider, stdout, field)
}

pub fn read_profile_document(
    profile: &crate::plan::AgentProfile,
    stdout: &Path,
    field: &str,
) -> Result<Value, String> {
    read_with_decoder(
        Decoder::for_profile(profile, 1024 * 1024),
        profile.parser,
        stdout,
        field,
    )
}

fn read_with_decoder(
    mut decoder: Decoder,
    provider: Provider,
    stdout: &Path,
    field: &str,
) -> Result<Value, String> {
    let mut file = fs::File::open(stdout).map_err(|e| e.to_string())?;
    let mut bytes = [0; 8192];
    let mut document = None;
    let mut text = String::new();
    let mut terminal = false;
    loop {
        let n = file.read(&mut bytes).map_err(|e| e.to_string())?;
        let batch = if n == 0 {
            decoder.finish()?
        } else {
            decoder.push(&bytes[..n])?
        };
        for event in batch {
            if event.kind == EventKind::Error {
                return Err(format!(
                    "coordinator reported an error: {}",
                    event_error(&event.payload)
                ));
            }
            terminal |= event.kind == EventKind::Result;
            let value = match provider {
                Provider::Claude if event.kind == EventKind::Result => event
                    .payload
                    .get("structured_output")
                    .or_else(|| event.payload.get("result")),
                Provider::Codex
                    if event.payload["type"] == "item.completed"
                        && event.payload.pointer("/item/type").and_then(Value::as_str)
                            == Some("agent_message") =>
                {
                    event.payload.pointer("/item/text")
                }
                Provider::OpenCode if event.payload["type"] == "text" => {
                    event.payload.pointer("/part/text")
                }
                Provider::Custom if event.kind == EventKind::Result => {
                    event.payload.get("payload").or(Some(&event.payload))
                }
                _ => None,
            };
            if let Some(value) = value {
                if let Some(chunk) = value.as_str() {
                    if chunk.len() > 64 * 1024 {
                        return Err("coordinator document exceeds 64 KiB".into());
                    }
                    if provider == Provider::OpenCode {
                        if text.len().saturating_add(chunk.len()) > 64 * 1024 {
                            return Err("coordinator document exceeds 64 KiB".into());
                        }
                        text.push_str(chunk);
                    } else if let Some(json) = json_reply(chunk, field) {
                        document = Some(json);
                    }
                } else if value.get(field).is_some() {
                    if serde_json::to_vec(value).map_err(|e| e.to_string())?.len() > 64 * 1024 {
                        return Err("coordinator document exceeds 64 KiB".into());
                    }
                    document = Some(value.clone());
                }
            }
        }
        if n == 0 {
            break;
        }
    }
    if provider == Provider::OpenCode && !text.is_empty() {
        document = json_reply(&text, field);
        terminal = document.is_some();
    }
    if !terminal {
        return Err("coordinator did not return successful final output".into());
    }
    document.ok_or_else(|| format!("coordinator response contains no valid JSON {field} document"))
}
