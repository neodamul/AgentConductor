use crate::{
    plan::AgentProfile,
    protocol::{Decoder, EventKind, Provider},
};
use serde_json::Value;
use std::{fs, io::Read, path::Path};

/// The whole reply as JSON, or the first fenced JSON block that carries `field`.
fn json_reply(text: &str, field: &str) -> Option<Value> {
    let has = |value: &Value| value.get(field).is_some();
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
    .find_map(|pointer| payload.pointer(pointer).and_then(Value::as_str))
    .map(str::to_owned)
    .unwrap_or_else(|| payload.to_string());
    let text = payload["api_error_status"]
        .as_i64()
        .map_or(text.clone(), |status| format!("{text} (HTTP {status})"));
    text.chars().take(2000).collect()
}

pub fn read_document(provider: Provider, stdout: &Path, field: &str) -> Result<Value, String> {
    read_with_decoder(provider, Decoder::new(provider, 1024 * 1024), stdout, field)
}

pub fn read_document_profile(
    profile: &AgentProfile,
    stdout: &Path,
    field: &str,
) -> Result<Value, String> {
    read_with_decoder(
        profile.parser,
        Decoder::for_profile(profile, 1024 * 1024),
        stdout,
        field,
    )
}

fn read_with_decoder(
    provider: Provider,
    mut decoder: Decoder,
    stdout: &Path,
    field: &str,
) -> Result<Value, String> {
    let mut file = fs::File::open(stdout).map_err(|e| e.to_string())?;
    let mut bytes = [0; 8192];
    let mut document = None;
    let mut text = String::new();
    let mut terminal = false;
    loop {
        let count = file.read(&mut bytes).map_err(|e| e.to_string())?;
        let events = if count == 0 {
            decoder.finish()?
        } else {
            decoder.push(&bytes[..count])?
        };
        for event in events {
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
        if count == 0 {
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
