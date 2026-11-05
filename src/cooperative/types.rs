//! The persisted shapes of a cooperative directive and its acknowledgments.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Address {
    pub run_id: String,
    pub plan_digest: String,
    pub task: String,
    pub agent: String,
    pub session_id: String,
    pub attempt: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Rework,
    Pause,
    Handoff,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Issue {
    pub revision: String,
    pub task: String,
    pub action: Action,
    pub instruction: String,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AckState {
    Accepted,
    Completed,
    Rejected,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Acknowledgment {
    pub id: String,
    pub address: Address,
    pub state: AckState,
    pub reported_at_ms: u64,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Pending,
    Accepted,
    Completed,
    Rejected,
    Expired,
    Superseded,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Directive {
    pub id: String,
    pub address: Address,
    pub action: Action,
    pub instruction: String,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
    pub state: State,
    pub acknowledgments: Vec<Acknowledgment>,
}
