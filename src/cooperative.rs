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

/// Routing context only: never creates a directive, approval or task transition.
pub fn worker_address(store: &mut Store, task: &str) -> Result<Address, String> {
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let r = report(&tx)?;
    let spec = r
        .plan
        .tasks
        .iter()
        .find(|t| t.id == task)
        .ok_or("unknown cooperative task")?;
    let profile = r
        .plan
        .agents
        .iter()
        .find(|p| p.id == spec.agent)
        .ok_or("unknown cooperative profile")?;
    if profile.cooperative != crate::plan::CooperativeMode::CheckpointV1 {
        return Err("worker did not register checkpoint_v1".into());
    }
    r.plan.validate()?;
    let a = address(&r, task, namespace(&tx)?)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(a)
}

/// Called by an operator/controller after reviewing this exact workflow revision.
/// This queues a scoped request only; it does not alter the task, permissions, or process.
pub fn issue(store: &mut Store, request: Issue, now: u64) -> Result<Directive, String> {
    validate_text(&request.instruction, 8192)?;
    if now == 0
        || request.issued_at_ms == 0
        || request.issued_at_ms > now
        || now - request.issued_at_ms > 300_000
        || request.expires_at_ms <= now
        || request.expires_at_ms.saturating_sub(request.issued_at_ms) > 86_400_000
    {
        return Err("directive lifetime must be positive and at most 24 hours".into());
    }
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let r = report(&tx)?;
    if workflow::revision(&r)? != request.revision {
        return Err("reviewed workflow revision changed".into());
    }
    // Namespace survives restart and differs even for identical plans and reused CLI session IDs.
    let run_id = namespace(&tx)?;
    let a = address(&r, &request.task, run_id)?;
    if let Some(mut prior) = active(&tx, &a)? {
        expire(&tx, &mut prior, now)?;
        if prior.address == a
            && prior.state == State::Completed
            && matches!(prior.action, Action::Pause | Action::Handoff)
        {
            return Err(
                "attempt relinquished at checkpoint; resume or reassign before issuing more work"
                    .into(),
            );
        }
        if prior.address != a && matches!(prior.state, State::Pending | State::Accepted) {
            // A reviewed new binding invalidates old requests, but does not stop their process.
            prior.state = State::Superseded;
            save(&tx, &prior)?;
        }
        if matches!(prior.state, State::Pending | State::Accepted) {
            return Err("addressed attempt already has an outstanding directive".into());
        }
    }
    let mut d = Directive {
        id: String::new(),
        address: a,
        action: request.action,
        instruction: request.instruction,
        issued_at_ms: request.issued_at_ms,
        expires_at_ms: request.expires_at_ms,
        state: State::Pending,
        acknowledgments: vec![],
    };
    d.id = identity(&d)?;
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE key=?1)",
            [format!("cooperative_directive/{}", d.id)],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if exists {
        return Err("directive already exists; inspect its recorded outcome".into());
    }
    save(&tx, &d)?;
    tx.execute(
        "INSERT INTO documents VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![
            format!(
                "cooperative_active/{}/{}",
                d.address.task, d.address.attempt
            ),
            serde_json::to_string(&d.id).map_err(|e| e.to_string())?
        ],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(d)
}
/// Polling may durably expire pending requests; it is not a read-only status query.
pub fn poll(store: &mut Store, a: &Address, now: u64) -> Result<Vec<Directive>, String> {
    if now == 0 {
        return Err("invalid cooperative observation time".into());
    }
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    validate_address(&tx, a)?;
    let mut result = vec![];
    if let Some(mut d) = active(&tx, a)? {
        if d.address != *a {
            return Err("outstanding directive belongs to another assignment".into());
        }
        expire(&tx, &mut d, now)?;
        if matches!(d.state, State::Pending | State::Accepted) {
            result.push(d);
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

/// The caller must prove process quiescence before interpreting this as an exit outcome.
pub fn exit_directive(
    store: &mut Store,
    a: &Address,
    now: u64,
) -> Result<Option<Directive>, String> {
    if now == 0 {
        return Err("invalid cooperative observation time".into());
    }
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    validate_address(&tx, a)?;
    let mut result = active(&tx, a)?;
    if let Some(d) = &mut result {
        if d.address != *a {
            return Err("exit directive belongs to another assignment".into());
        }
        expire(&tx, d, now)?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}
pub fn acknowledge(store: &mut Store, ack: Acknowledgment, now: u64) -> Result<Directive, String> {
    validate_text(&ack.reason, 4096)?;
    if now == 0 || ack.reported_at_ms == 0 || ack.reported_at_ms > now {
        return Err("invalid acknowledgment time".into());
    }
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    validate_address(&tx, &ack.address)?;
    let mut d = get(&tx, &ack.id)?;
    if d.address != ack.address || ack.reported_at_ms < d.issued_at_ms {
        return Err("acknowledgment address or time does not match directive".into());
    }
    if let Some(prior) = d.acknowledgments.iter().find(|a| a.state == ack.state) {
        if prior != &ack {
            return Err("conflicting duplicate acknowledgment".into());
        }
        tx.commit().map_err(|e| e.to_string())?;
        return Ok(d);
    }
    expire(&tx, &mut d, now)?;
    if d.state == State::Expired {
        tx.commit().map_err(|e| e.to_string())?;
        return Err("cooperative directive expired before acceptance".into());
    }
    let next = match (&d.state, &ack.state) {
        (State::Pending, AckState::Accepted) => State::Accepted,
        (State::Pending | State::Accepted, AckState::Rejected) => State::Rejected,
        (State::Accepted, AckState::Completed) => State::Completed,
        _ => return Err("invalid cooperative acknowledgment transition".into()),
    };
    if d.acknowledgments
        .last()
        .is_some_and(|a| a.reported_at_ms > ack.reported_at_ms)
    {
        return Err("acknowledgment time moved backwards".into());
    }
    d.state = next;
    d.acknowledgments.push(ack);
    save(&tx, &d)?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(d)
}

mod continuation;
pub use continuation::{Continuation, prepare_continuation};

mod progress;
pub mod route;
pub(crate) use progress::report_progress_from;
pub use progress::{Progress, report_progress};

pub mod prompt;
