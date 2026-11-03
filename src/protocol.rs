mod provider;
mod text_output;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Acio,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Claude,
    Codex,
    #[serde(rename = "opencode")]
    OpenCode,
    #[default]
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Session,
    Progress,
    Tool,
    Result,
    Usage,
    Warning,
    Error,
    Denied,
    Unknown,
}

#[derive(Debug, Serialize)]
pub struct Event {
    pub seq: u64,
    pub raw_offset: u64,
    pub session_id: Option<String>,
    pub kind: EventKind,
    pub payload: Value,
}

pub struct Decoder {
    provider: Provider,
    event_types: std::collections::BTreeMap<String, EventKind>,
    session_id_pointer: Option<String>,
    result_pointer: Option<String>,
    error_condition: Option<crate::plan::JsonCondition>,
    denial: Option<crate::plan::DenialCondition>,
    text_mode: bool,
    acio: bool,
    limit: usize,
    buffer: Vec<u8>,
    offset: u64,
    seq: u64,
    session_id: Option<String>,
}

impl Decoder {
    pub fn new(provider: Provider, limit: usize) -> Self {
        Self {
            provider,
            event_types: Default::default(),
            session_id_pointer: None,
            result_pointer: None,
            error_condition: None,
            denial: None,
            text_mode: false,
            acio: false,
            limit,
            buffer: vec![],
            offset: 0,
            seq: 0,
            session_id: None,
        }
    }

    pub fn for_profile(profile: &crate::plan::AgentProfile, limit: usize) -> Self {
        let mut decoder = Self::new(profile.parser, limit);
        decoder.acio = profile.protocol == Some(Protocol::Acio);
        if profile.parser == Provider::Custom {
            decoder.event_types.clone_from(&profile.event_types);
            decoder
                .session_id_pointer
                .clone_from(&profile.session_id_pointer);
            decoder.result_pointer.clone_from(&profile.result_pointer);
            decoder.error_condition.clone_from(&profile.error_condition);
            decoder.denial.clone_from(&profile.denial_condition);
            decoder.text_mode = profile.on_invalid_json == crate::plan::InvalidJson::Text;
        }
        decoder
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Event>, String> {
        let mut events = vec![];
        for part in bytes.split_inclusive(|b| *b == b'\n') {
            if self.buffer.len().saturating_add(part.len()) > self.limit {
                return Err(format!(
                    "JSONL line exceeds {} bytes at offset {}",
                    self.limit, self.offset
                ));
            }
            self.buffer.extend_from_slice(part);
            if part.last() == Some(&b'\n') {
                events.extend(self.finish()?);
            }
        }
        Ok(events)
    }

    pub fn finish(&mut self) -> Result<Vec<Event>, String> {
        if self.buffer.is_empty() {
            return Ok(vec![]);
        }
        let bytes = std::mem::take(&mut self.buffer);
        let start = self.offset;
        self.offset += bytes.len() as u64;
        if bytes.iter().all(u8::is_ascii_whitespace) {
            return Ok(vec![]);
        }
        // A stream-leading UTF-8 signature is transport metadata, not part of the JSONL record.
        let json = if start == 0 {
            bytes
                .as_slice()
                .strip_prefix(b"\xef\xbb\xbf")
                .unwrap_or(&bytes)
        } else {
            &bytes
        };
        let payload: Value = match serde_json::from_slice(json) {
            Ok(payload) => payload,
            Err(_) if self.text_mode && self.provider == Provider::Custom => {
                let event = text_output::event(self.seq, start, self.session_id.clone(), json);
                self.seq += 1;
                return Ok(vec![event]);
            }
            Err(error) => {
                return Err(format!(
                    "invalid JSONL at offset {start}: {error}; the line was: {}",
                    text_output::excerpt(json)
                ));
            }
        };
        if !payload.is_object() {
            return Err(format!("event must be a JSON object at offset {start}"));
        }
        let session = if self.acio {
            payload
                .get("session_id")
                .or_else(|| payload.pointer("/data/session_id"))
        } else {
            self.session_id_pointer.as_deref().map_or_else(
                || match self.provider {
                    Provider::Codex => payload.get("thread_id"),
                    Provider::OpenCode => payload
                        .get("sessionID")
                        .or_else(|| payload.pointer("/part/sessionID")),
                    _ => payload.get("session_id"),
                },
                |pointer| {
                    if pointer.is_empty() {
                        Some(&payload)
                    } else {
                        payload.pointer(pointer)
                    }
                },
            )
        };
        if let Some(id) = session.and_then(Value::as_str) {
            self.session_id = Some(id.into());
        }
        let event_type = payload.get("type").and_then(Value::as_str).unwrap_or("");
        let configured_error = self.provider == Provider::Custom
            && self.error_condition.as_ref().is_some_and(|condition| {
                payload.pointer(&condition.pointer) == Some(&condition.equals)
            });
        let denied = self.provider == Provider::Custom
            && !self.acio
            && self
                .denial
                .as_ref()
                .is_some_and(|condition| condition.matches(&payload));
        let kind = if self.acio {
            provider::acio_kind(&payload)
        } else if denied {
            EventKind::Denied
        } else if configured_error {
            EventKind::Error
        } else if self.provider == Provider::Custom
            && let Some(kind) = self.event_types.get(event_type)
        {
            *kind
        } else {
            match self.provider {
                Provider::Claude
                    if payload
                        .get("api_error")
                        .is_some_and(|v| !v.is_null() && v.as_str() != Some(""))
                        || payload["api_error_status"]
                            .as_i64()
                            .is_some_and(|n| n >= 400) =>
                {
                    EventKind::Error
                }
                Provider::Claude => match event_type {
                    "system" if payload["subtype"] == "init" => EventKind::Session,
                    "result" if payload["is_error"] == true => EventKind::Error,
                    "result" => EventKind::Result,
                    "assistant" | "user" | "stream_event" => EventKind::Progress,
                    "rate_limit_event" => EventKind::Usage,
                    "error" => EventKind::Error,
                    _ => EventKind::Unknown,
                },
                Provider::Codex => match event_type {
                    "error" if provider::codex_reconnecting(&payload) => EventKind::Warning,
                    "item.completed"
                        if payload.pointer("/item/type").and_then(Value::as_str)
                            == Some("error") =>
                    {
                        EventKind::Warning
                    }
                    "thread.started" => EventKind::Session,
                    "turn.completed" => EventKind::Result,
                    "turn.failed" | "error" => EventKind::Error,
                    "item.started" | "item.updated" | "item.completed"
                        if matches!(
                            payload.pointer("/item/type").and_then(Value::as_str),
                            Some("command_execution" | "mcp_tool_call" | "file_change")
                        ) =>
                    {
                        EventKind::Tool
                    }
                    "turn.started" | "item.started" | "item.updated" | "item.completed" => {
                        EventKind::Progress
                    }
                    _ => EventKind::Unknown,
                },
                Provider::OpenCode => match event_type {
                    "text" | "step_start" | "step_finish" => EventKind::Progress,
                    "tool_use" => EventKind::Tool,
                    "error" => EventKind::Error,
                    _ => EventKind::Unknown,
                },
                Provider::Custom => match event_type {
                    "session" => EventKind::Session,
                    "progress" | "phase" => EventKind::Progress,
                    "tool" => EventKind::Tool,
                    "result" => EventKind::Result,
                    "usage" => EventKind::Usage,
                    "error" => EventKind::Error,
                    _ => EventKind::Unknown,
                },
            }
        };
        let payload = if kind == EventKind::Result
            && event_type == "result"
            && let Some(pointer) = &self.result_pointer
        {
            payload
                .pointer(pointer)
                .cloned()
                .ok_or("configured result JSON Pointer was not found")?
        } else {
            payload
        };
        let event = Event {
            seq: self.seq,
            raw_offset: start,
            session_id: self.session_id.clone(),
            kind,
            payload,
        };
        self.seq += 1;
        Ok(vec![event])
    }
}
