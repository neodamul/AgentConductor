use serde::Serialize;
use serde_json::Value;

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

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Session,
    Progress,
    Tool,
    Result,
    Usage,
    Warning,
    Error,
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
            limit,
            buffer: vec![],
            offset: 0,
            seq: 0,
            session_id: None,
        }
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
        let payload: Value = serde_json::from_slice(&bytes)
            .map_err(|e| format!("invalid JSONL at offset {start}: {e}"))?;
        if !payload.is_object() {
            return Err(format!("event must be a JSON object at offset {start}"));
        }
        let session = match self.provider {
            Provider::Codex => payload.get("thread_id"),
            Provider::OpenCode => payload
                .get("sessionID")
                .or_else(|| payload.pointer("/part/sessionID")),
            _ => payload.get("session_id"),
        };
        if let Some(id) = session.and_then(Value::as_str) {
            self.session_id = Some(id.into());
        }
        let event_type = payload.get("type").and_then(Value::as_str).unwrap_or("");
        let kind = match self.provider {
            Provider::Claude => match event_type {
                "system" if payload["subtype"] == "init" => EventKind::Session,
                "result" if payload["is_error"] == true => EventKind::Error,
                "result" => EventKind::Result,
                "assistant" | "user" | "stream_event" => EventKind::Progress,
                "error" => EventKind::Error,
                _ => EventKind::Unknown,
            },
            Provider::Codex => match event_type {
                "item.completed"
                    if payload.pointer("/item/type").and_then(Value::as_str) == Some("error") =>
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
