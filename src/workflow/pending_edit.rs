//! Agent and model changes for a task that has not started while the run is active (#103).
//! The running scheduler owns the report, so the change is queued like active feedback and
//! applied by the scheduler before the task is launched.
use super::Report;
use crate::state::{Store, record_transaction};
use rusqlite::{OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;

const KEY: &str = "pending_edit";

#[derive(Default, Serialize, Deserialize)]
struct Edit {
    agent: Option<String>,
    model: Option<String>,
}

/// The plan with the edit applied, or why it is not valid.
fn candidate(report: &Report, index: usize, edit: &Edit) -> Result<crate::plan::Plan, String> {
    let mut plan = report.plan.clone();
    let task = &mut plan.tasks[index];
    if let Some(agent) = &edit.agent {
        if &task.agent != agent {
            task.model = None;
            task.fallback = None;
        }
        task.agent = agent.clone();
    }
    if let Some(model) = &edit.model {
        task.model = Some(model.clone());
    }
    let agent = plan
        .agents
        .iter()
        .find(|a| a.id == plan.tasks[index].agent)
        .ok_or("unknown target agent")?;
    if let Some(model) = &plan.tasks[index].model {
        agent.validate_model(model)?;
    }
    plan.validate()?;
    Ok(plan)
}

/// Queues the edit for a pending task of a running run. False when the run is not running,
/// so the caller edits the stopped run directly.
pub(super) fn queue(
    dir: &Path,
    task: &str,
    agent: Option<&str>,
    model: Option<&str>,
) -> Result<bool, String> {
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let value: String = tx
        .query_row(
            "SELECT value FROM documents WHERE key='workflow'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let report: Report = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    if report.state != "running" {
        return Ok(false);
    }
    let index = report
        .tasks
        .iter()
        .position(|t| t.id == task)
        .ok_or("unknown task/session ID")?;
    // GitHub #1054: a task waiting for its account's usage limit has not run this attempt either.
    if !matches!(
        report.tasks[index].status.as_str(),
        "pending" | "waiting_availability"
    ) {
        return Err(
            "only a task that has not started can be changed while the run is active".into(),
        );
    }
    let key = format!("{KEY}/{task}");
    let prior: Option<String> = tx
        .query_row("SELECT value FROM documents WHERE key=?1", [&key], |r| {
            r.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    let mut edit: Edit = prior
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default();
    if let Some(agent) = agent {
        edit.agent = Some(agent.into());
        edit.model = None;
    }
    if let Some(model) = model {
        edit.model = Some(model.into());
    }
    candidate(&report, index, &edit)?;
    let value = serde_json::to_value(&edit).map_err(|e| e.to_string())?;
    record_transaction(
        &tx,
        &key,
        &value,
        "Operator change queued for a task that has not started",
    )?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(true)
}

/// Whether an edit waits to be applied, so the scheduler applies it without waiting for a worker.
pub(super) fn queued(store: &Store) -> Result<bool, String> {
    store
        .0
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM documents WHERE key LIKE 'pending_edit/%')",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())
}

/// Applies queued edits to tasks that have not started; the scheduler calls this before
/// deciding what to launch. An edit that is no longer valid is dropped and recorded.
/// GitHub #1054: a new agent is recorded as a handoff, so the task's activity shows it; a task that
/// already ran or waits for its account's usage limit hands its work over like a reassignment.
pub(super) fn apply(store: &mut Store, report: &mut Report, dir: &Path) -> Result<(), String> {
    for index in 0..report.tasks.len() {
        if !matches!(
            report.tasks[index].status.as_str(),
            "pending" | "waiting_availability"
        ) {
            continue;
        }
        let key = format!("{KEY}/{}", report.tasks[index].id);
        let value: Option<String> = store
            .0
            .query_row("SELECT value FROM documents WHERE key=?1", [&key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())?;
        let Some(edit) = value.and_then(|v| serde_json::from_str::<Edit>(&v).ok()) else {
            continue;
        };
        let from = report.plan.tasks[index].agent.clone();
        let mut applied = candidate(report, index, &edit).map(|_| ());
        store
            .0
            .execute("DELETE FROM documents WHERE key=?1", [&key])
            .map_err(|e| e.to_string())?;
        let moved = edit.agent.as_ref().filter(|agent| **agent != from);
        if let (Ok(()), Some(agent)) = (&applied, moved) {
            let task = &report.tasks[index];
            if task.status == "waiting_availability" || task.attempt > 0 {
                applied = super::assignment::transfer_role(
                    report,
                    dir,
                    index,
                    agent,
                    edit.model.as_deref(),
                    store,
                    "Operator reassigned a waiting role; target uses a new provider session",
                );
            } else {
                report.plan = candidate(report, index, &edit)?;
                store.record(
                    "handoff",
                    &serde_json::json!({"task":task.id,"from":from,"to":agent,"queued":true}),
                    "Queued reassignment applied before the task started",
                )?;
            }
        } else if let Ok(()) = &applied {
            report.plan = candidate(report, index, &edit)?;
        }
        store.record(
            "pending_edit_applied",
            &serde_json::json!({"task":report.tasks[index].id,"agent":edit.agent,"model":edit.model,"error":applied.err()}),
            "Queued agent or model change applied before the task started",
        )?;
    }
    Ok(())
}
