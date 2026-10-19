use super::Report;
use crate::state::{Store, record_transaction};
use rusqlite::{OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::path::Path;

const KEY: &str = "active_feedback";
const MAX_BYTES: usize = 16 * 1024;

#[derive(Serialize, Deserialize)]
struct Queued {
    attempt: u32,
    messages: Vec<String>,
}

fn read_report(tx: &rusqlite::Transaction<'_>) -> Result<Report, String> {
    let value: String = tx
        .query_row(
            "SELECT value FROM documents WHERE key='workflow'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&value).map_err(|e| e.to_string())
}

pub(super) fn queue(dir: &Path, task: &str, text: &str) -> Result<bool, String> {
    if text.trim().is_empty() || text.len() > MAX_BYTES || text.contains('\0') {
        return Err("feedback must contain 1..16384 bytes".into());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let report = read_report(&tx)?;
    if report.state != "running" {
        return Ok(false);
    }
    let current = report
        .tasks
        .iter()
        .find(|entry| entry.id == task)
        .ok_or("unknown task/session ID")?;
    if current.status != "running" || current.attempt == 0 {
        return Err("feedback can only be sent to a task with a running attempt".into());
    }
    let key = format!("{KEY}/{task}");
    let prior: Option<String> = tx
        .query_row("SELECT value FROM documents WHERE key=?1", [&key], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    let mut queued = match prior {
        Some(value) => serde_json::from_str::<Queued>(&value).map_err(|e| e.to_string())?,
        None => Queued {
            attempt: current.attempt,
            messages: vec![],
        },
    };
    if queued.attempt > current.attempt {
        return Err("queued feedback belongs to a later attempt".into());
    }
    if queued.attempt < current.attempt {
        queued.attempt = current.attempt;
    }
    queued.messages.push(text.into());
    let bytes = queued.messages.iter().map(String::len).sum::<usize>()
        + queued.messages.len().saturating_sub(1) * 2;
    if bytes > MAX_BYTES {
        return Err("queued feedback exceeds 16384 bytes for this attempt".into());
    }
    let value = serde_json::to_value(queued).map_err(|e| e.to_string())?;
    record_transaction(
        &tx,
        &key,
        &value,
        "Operator feedback queued for the next safe task checkpoint",
    )?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(true)
}

pub(in crate::workflow) fn settle(
    store: &mut Store,
    report: &mut Report,
    index: usize,
) -> Result<(), String> {
    let task = &mut report.tasks[index];
    let key = format!("{KEY}/{}", task.id);
    let tx = store
        .0
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| e.to_string())?;
    let value: Option<String> = tx
        .query_row("SELECT value FROM documents WHERE key=?1", [&key], |row| {
            row.get(0)
        })
        .optional()
        .map_err(|e| e.to_string())?;
    if let Some(value) = value {
        let queued: Queued = serde_json::from_str(&value).map_err(|e| e.to_string())?;
        if queued.attempt <= task.attempt {
            task.feedback = Some(queued.messages.join("\n\n"));
            if !matches!(task.status.as_str(), "stopped" | "waiting_availability") {
                task.status = "pending".into();
            }
            task.error = Some(
                "Feedback queued during this attempt; it will be applied after the owned process exits."
                    .into(),
            );
            let event = format!("active_feedback_applied/{}/{}", task.id, task.attempt);
            record_transaction(
                &tx,
                &event,
                &serde_json::json!({"task":task.id,"attempt":task.attempt,"feedback":task.feedback}),
                "Queued feedback attached to the next safe task checkpoint",
            )?;
            tx.execute("DELETE FROM documents WHERE key=?1", [&key])
                .map_err(|e| e.to_string())?;
        }
    }
    let serialized = serde_json::to_string(report).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO documents(key,value) VALUES('workflow',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [serialized],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
