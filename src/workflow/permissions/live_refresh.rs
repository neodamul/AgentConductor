use super::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const QUEUE_FILE: &str = "tool-refresh.request";
const MAX_QUEUE_BYTES: u64 = 4096;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QueuedRefresh {
    request_id: String,
    queued_at_ms: u128,
}

/// Queues a refresh for the coordinator currently holding the run lease.
/// This writes only a bounded request; it cannot change or approve a receipt.
pub fn queue_live_tool_refresh(dir: &Path, id: &str) -> Result<(), String> {
    let report = load(dir)?;
    if report.state != "running" {
        return Err("live tool refresh requires a running coordinator".into());
    }
    let ledger = crate::gates::Ledger::open(&dir.join("state.sqlite"))?;
    let receipt = ledger.receipt(id)?.ok_or("unknown tool request SHA-256")?;
    let task = report
        .tasks
        .iter()
        .find(|task| task.id == receipt.context.task)
        .ok_or("tool request task is not in this run")?;
    if task.status != "running"
        || task.session_id.as_deref() != Some(receipt.context.session.as_str())
        || task.approval_epoch != receipt.context.epoch
        || !matches!(receipt.status.as_str(), "pending" | "approved")
        || !active_process(&task.artifacts)
    {
        return Err("tool request is no longer pending in its active worker".into());
    }
    let latest = ledger.requests()?.into_iter().find(|candidate| {
        candidate.context.task == receipt.context.task
            && candidate.context.session == receipt.context.session
            && candidate.request == receipt.request
            && candidate.status != "superseded"
    });
    if latest.as_ref().map(|candidate| candidate.id.as_str()) != Some(id) {
        return Err("tool request is not the current decision for its worker".into());
    }
    cleanup_temporary_queues(dir)?;
    let path = dir.join(QUEUE_FILE);
    let temporary = dir.join(format!(
        "tool-refresh-{}.pending",
        crate::native_guard::session_id()?
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| error.to_string())?;
    let request = QueuedRefresh {
        request_id: id.into(),
        queued_at_ms: now(),
    };
    let written = serde_json::to_writer(&mut file, &request)
        .map_err(|error| error.to_string())
        .and_then(|()| file.sync_all().map_err(|error| error.to_string()));
    drop(file);
    if let Err(error) = written {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    let published = fs::hard_link(&temporary, &path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "a tool refresh is already queued for this run".to_string()
        } else {
            error.to_string()
        }
    });
    let _ = fs::remove_file(temporary);
    published
}

/// Applies a queued refresh while the scheduler owns the coordinator lease.
/// It advances both the in-memory and persisted approval epoch before rotating
/// the receipt, so neither stale approval nor stale worker state can authorize it.
pub async fn apply_queued_tool_refresh(
    dir: &Path,
    report: &mut Report,
    store: &mut Store,
    engine: &Path,
) -> Result<bool, String> {
    let path = dir.join(QUEUE_FILE);
    let request = match read_queue(&path) {
        Err(error) => {
            reject_queue(dir, store, "invalid", &error)?;
            return Ok(false);
        }
        Ok(None) => return Ok(false),
        Ok(Some(request)) => request,
    };
    let receipt =
        crate::gates::Ledger::open(&dir.join("state.sqlite"))?.receipt(&request.request_id)?;
    let Some(receipt) = receipt else {
        reject_queue(dir, store, &request.request_id, "receipt disappeared")?;
        return Ok(false);
    };
    let Some(index) = report
        .tasks
        .iter()
        .position(|task| task.id == receipt.context.task)
    else {
        reject_queue(dir, store, &request.request_id, "task disappeared")?;
        return Ok(false);
    };
    let task = &report.tasks[index];
    if report.state != "running"
        || task.status != "running"
        || task.session_id.as_deref() != Some(receipt.context.session.as_str())
        || task.approval_epoch != receipt.context.epoch
        || !matches!(receipt.status.as_str(), "pending" | "approved")
        || !active_process(&task.artifacts)
    {
        reject_queue(
            dir,
            store,
            &request.request_id,
            "worker exited before refresh",
        )?;
        return Ok(false);
    }
    let prior_epoch = task.approval_epoch;
    report.tasks[index].approval_epoch = prior_epoch
        .checked_add(1)
        .ok_or("approval epoch exhausted")?;
    save(store, dir, report)?;
    let context =
        match verify_tool_context(dir, &receipt.context.task, &receipt.request, engine).await {
            Ok(context) => context,
            Err(error) => {
                report.tasks[index].approval_epoch = prior_epoch;
                save(store, dir, report)?;
                reject_queue(dir, store, &request.request_id, &error)?;
                return Ok(false);
            }
        };
    if !active_process(&report.tasks[index].artifacts) {
        report.tasks[index].approval_epoch = prior_epoch;
        save(store, dir, report)?;
        reject_queue(
            dir,
            store,
            &request.request_id,
            "worker exited during refresh",
        )?;
        return Ok(false);
    }
    let fresh = match crate::gates::Ledger::open(&dir.join("state.sqlite"))?.refresh(
        &request.request_id,
        &context,
        &receipt.request,
        (now() / 1000) as i64,
    ) {
        Ok(fresh) => fresh,
        Err(error) => {
            report.tasks[index].approval_epoch = prior_epoch;
            save(store, dir, report)?;
            reject_queue(dir, store, &request.request_id, &error)?;
            return Ok(false);
        }
    };
    report.tasks[index].tool_request = Some(fresh.clone());
    store.record(
        "tool_refresh_applied",
        &serde_json::json!({"old_request":request.request_id,"new_request":fresh,"task":receipt.context.task,"epoch":report.tasks[index].approval_epoch,"queued_at_ms":request.queued_at_ms}),
        "Coordinator refreshed the exact pending request; replacement still requires human approval",
    )?;
    save(store, dir, report)?;
    fs::remove_file(path).map_err(|error| error.to_string())?;
    Ok(true)
}

fn read_queue(path: &Path) -> Result<Option<QueuedRefresh>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_QUEUE_BYTES
    {
        return Err("tool refresh queue file is invalid".into());
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| format!("invalid tool refresh queue: {error}"))
}

fn cleanup_temporary_queues(dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("tool-refresh-") && name.ends_with(".pending") {
            let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
            let abandoned = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.elapsed().ok())
                .is_some_and(|age| age > std::time::Duration::from_secs(3600));
            if abandoned && metadata.is_file() && !metadata.file_type().is_symlink() {
                fs::remove_file(entry.path()).map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

fn active_process(artifacts: &Path) -> bool {
    let path = artifacts.join("process/owner.json");
    let Ok(metadata) = fs::symlink_metadata(&path) else {
        return false;
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 4096 {
        return false;
    }
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<crate::process::Ownership>(&bytes).ok())
        .is_some_and(|owner| owner.phase == "running" && owner.pid.is_some())
}

fn reject_queue(dir: &Path, store: &mut Store, id: &str, reason: &str) -> Result<(), String> {
    store.record(
        "tool_refresh_rejected",
        &serde_json::json!({"request_id":id,"reason":reason}),
        "Coordinator rejected a stale or invalid live refresh request",
    )?;
    match fs::remove_file(dir.join(QUEUE_FILE)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
