//! The wire protocols, providers and normalized events a decoder produces.
use serde::Serialize;
use serde_json::Value;

/// A wire protocol a custom profile speaks instead of a hand-written event mapping (GitHub #208).
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
    /// GitHub #341: the CLI refused a tool (a profile's `denial_condition` matched).
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
