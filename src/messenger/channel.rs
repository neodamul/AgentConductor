//! #515: the channel-neutral contract. A channel only transports; the core composes the notice
//! text and enforces sender authorization, request-code binding (DD-32) and the audit record.
use super::{Event, Settings, inbound::Reply};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::Path;

/// What a channel can do. A request for a missing capability is refused by the core with a hint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    /// Outbound notices for run events.
    pub notice: bool,
    /// Decision replies bound to a request code (R-43 phase 2).
    pub replies: bool,
    /// Free conversation with the coordinator (planned, #519).
    pub conversation: bool,
}

/// Everything a channel needs to read new replies.
pub struct Fetch<'a> {
    pub workspace: &'a Path,
    pub settings: &'a Settings,
    /// The newest reply timestamp already handled; empty on the first poll.
    pub cursor: &'a str,
}

pub trait Channel: Sync {
    /// The stable identifier stored in the workspace settings (`kind`).
    fn id(&self) -> &'static str;
    fn capabilities(&self) -> Capabilities;
    /// Whether the settings dialog offers a message body template.
    fn uses_template(&self) -> bool {
        false
    }
    /// The request body for a notice. `text` is composed by the core; `goal` is empty unless allowed.
    fn notice_body(
        &self,
        settings: &Settings,
        text: &str,
        event: &Event,
        goal: &str,
    ) -> Result<serde_json::Value, String>;
    /// New replies after `fetch.cursor` plus the lines of any refused ones.
    fn fetch_replies(&self, _fetch: &Fetch<'_>) -> Result<(Vec<Reply>, Vec<String>), String> {
        Err("this messenger channel cannot read replies".into())
    }
}

/// The one registration point: a new channel is one implementation and one entry here.
static REGISTRY: [&dyn Channel; 2] = [&super::slack::Slack, &super::webhook::Webhook];

pub fn channels() -> &'static [&'static dyn Channel] {
    &REGISTRY
}

pub fn find(id: &str) -> Option<&'static dyn Channel> {
    REGISTRY.iter().copied().find(|c| c.id() == id)
}

/// A registered channel's identifier, stored as the same text as before (`slack`, `webhook`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind(&'static str);

#[allow(non_upper_case_globals)]
impl Kind {
    pub const Slack: Kind = Kind("slack");
    pub const Webhook: Kind = Kind("webhook");
    pub fn id(self) -> &'static str {
        self.0
    }
    pub fn channel(self) -> &'static dyn Channel {
        find(self.0).expect("a Kind always names a registered channel")
    }
}

impl Default for Kind {
    fn default() -> Self {
        Kind::Slack
    }
}

impl Serialize for Kind {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.0)
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let id = String::deserialize(d)?;
        find(&id)
            .map(|c| Kind(c.id()))
            .ok_or_else(|| serde::de::Error::custom(format!("unknown messenger channel `{id}`")))
    }
}
