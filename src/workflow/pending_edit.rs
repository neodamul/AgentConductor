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
    if report.tasks[index].status != "pending" {
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

/// Applies queued edits to tasks that are still pending; the scheduler calls this before
/// deciding what to launch. An edit that is no longer valid is dropped and recorded.
pub(super) fn apply(store: &mut Store, report: &mut Report) -> Result<(), String> {
    for index in 0..report.tasks.len() {
        if report.tasks[index].status != "pending" {
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
        let applied = candidate(report, index, &edit);
        if let Ok(plan) = &applied {
            report.plan = plan.clone();
        }
        store
            .0
            .execute("DELETE FROM documents WHERE key=?1", [&key])
            .map_err(|e| e.to_string())?;
        store.record(
            "pending_edit_applied",
            &serde_json::json!({"task":report.tasks[index].id,"agent":edit.agent,"model":edit.model,"error":applied.err()}),
            "Queued agent or model change applied before the task started",
        )?;
    }
    Ok(())
}
