//! Durable addressed checkpoint directives. Memory replicas cannot grant execution permission.
use crate::{
    state::{Store, record_transaction},
    workflow::{self, Report},
};
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};
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
fn validate_text(s: &str, max: usize) -> Result<(), String> {
    if s.trim().is_empty() || s.len() > max || s.chars().any(|c| c == '\0') {
        Err("empty or oversized cooperative field".into())
    } else {
        Ok(())
    }
}
fn report(tx: &Transaction<'_>) -> Result<Report, String> {
    let value: String = tx
        .query_row(
            "SELECT value FROM documents WHERE key='workflow'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&value).map_err(|e| e.to_string())
}
fn address(report: &Report, task: &str, run_id: String) -> Result<Address, String> {
    let index = report
        .tasks
        .iter()
        .position(|t| t.id == task)
        .ok_or("unknown cooperative task")?;
    let task = &report.tasks[index];
    let spec = report
        .plan
        .tasks
        .get(index)
        .filter(|t| t.id == task.id)
        .ok_or("inconsistent cooperative task")?;
    if task.attempt == 0 || !matches!(task.status.as_str(), "running" | "waiting_approval") {
        return Err("cooperative task has no active addressed attempt".into());
    }
    let session_id = task
        .session_id
        .as_ref()
        .ok_or("cooperative task has no native session")?;
    validate_text(session_id, 256)?;
    Ok(Address {
        run_id,
        plan_digest: workflow::digest(
            &serde_json::to_vec(&report.plan).map_err(|e| e.to_string())?,
        ),
        task: task.id.clone(),
        agent: spec.agent.clone(),
        session_id: session_id.clone(),
        attempt: task.attempt,
    })
}
fn validate_address(tx: &Transaction<'_>, expected: &Address) -> Result<(), String> {
    let run_id: String = tx
        .query_row(
            "SELECT value FROM documents WHERE key='cooperative_run_id'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let run_id: String = serde_json::from_str(&run_id).map_err(|e| e.to_string())?;
    if address(&report(tx)?, &expected.task, run_id)? != *expected {
        return Err("cooperative address is stale or belongs to another session".into());
    }
    Ok(())
}
fn get(tx: &Transaction<'_>, id: &str) -> Result<Directive, String> {
    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid directive ID".into());
    }
    let value: String = tx
        .query_row(
            "SELECT value FROM documents WHERE key=?1",
            [format!("cooperative_directive/{id}")],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let record: Directive = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    if record.id != id || identity(&record)? != id {
        return Err("cooperative directive identity changed".into());
    }
    Ok(record)
}
fn identity(d: &Directive) -> Result<String, String> {
    let immutable = (
        &d.address,
        &d.action,
        &d.instruction,
        d.issued_at_ms,
        d.expires_at_ms,
    );
    Ok(workflow::digest(
        &serde_json::to_vec(&immutable).map_err(|e| e.to_string())?,
    ))
}
fn save(tx: &Transaction<'_>, record: &Directive) -> Result<(), String> {
    let value = serde_json::to_value(record).map_err(|e| e.to_string())?;
    // Include the bounded full envelope so a company CLI can locate and validate an exact ID.
    let summary = serde_json::to_string(
        &serde_json::json!({"protocol":"agentconductor.checkpoint.v1","directive":record}),
    )
    .map_err(|e| e.to_string())?;
    record_transaction(
        tx,
        &format!("cooperative_directive/{}", record.id),
        &value,
        &summary,
    )
}
fn active(tx: &Transaction<'_>, a: &Address) -> Result<Option<Directive>, String> {
    let value: Option<String> = tx
        .query_row(
            "SELECT value FROM documents WHERE key=?1",
            [format!("cooperative_active/{}/{}", a.task, a.attempt)],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    value
        .map(|s| {
            let id: String = serde_json::from_str(&s).map_err(|e| e.to_string())?;
            get(tx, &id)
        })
        .transpose()
}
fn expire(tx: &Transaction<'_>, d: &mut Directive, now: u64) -> Result<(), String> {
    if d.state == State::Pending && now >= d.expires_at_ms {
        d.state = State::Expired;
        save(tx, d)?;
    }
    Ok(())
}
fn namespace(tx: &Transaction<'_>) -> Result<String, String> {
    let existing: Option<String> = tx
        .query_row(
            "SELECT value FROM documents WHERE key='cooperative_run_id'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(value) = existing {
        return serde_json::from_str(&value).map_err(|e| e.to_string());
    }
    let id: String = tx
        .query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO documents VALUES('cooperative_run_id',?1)",
        [serde_json::to_string(&id).map_err(|e| e.to_string())?],
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

mod continuation;
mod ledger;
pub use continuation::{Continuation, prepare_continuation};
pub use ledger::{acknowledge, exit_directive, issue, poll, worker_address};

mod progress;
pub mod route;
pub(crate) use progress::report_progress_from;
pub use progress::{Progress, report_progress};

pub mod prompt;
