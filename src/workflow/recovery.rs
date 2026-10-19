use super::*;
mod resume;
pub use resume::{resume_availability_due, resume_due};

/// Durable request; the active coordinator consumes and audits it after draining its jobs.
pub fn stop(dir: &Path, reason: &str) -> Result<(), String> {
    let report = load(dir)?;
    if reason.trim().is_empty()
        || reason.len() > 16 * 1024
        || !matches!(
            report.state.as_str(),
            "running" | "ready" | "waiting_availability" | "waiting_tool_approval" | "stopped"
        )
    {
        return Err("stop needs a reason and an active, ready, or restricted run".into());
    }
    let stop_lease = crate::leases::Lease::acquire(&dir.join("stop.lock"))?;
    let temporary = dir.join(format!(
        "stop-{}.pending",
        crate::native_guard::session_id()?
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(
                &mut file,
                &serde_json::json!({"reason":reason,"requested_at_ms":now(),"request_id":crate::native_guard::session_id()?}),
            )
            .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    fs::rename(&temporary, dir.join("stop.request")).map_err(|e| e.to_string())?;
    drop(stop_lease);
    if let Ok(_lock) = Lock::acquire(dir) {
        let mut report = load(dir)?;
        if report.state == "running" || report.tasks.iter().any(|t| t.status == "running") {
            return Err("unknown process ownership; request retained for explicit recovery".into());
        }
        report.state = "stopped".into();
        let mut store = Store::open(dir.join("state.sqlite"))?;
        store.record(
            "stop",
            &serde_json::json!({"reason":reason}),
            "Human stopped an idle coordinator",
        )?;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Stop revoked candidate approval",
        )?;
        save(&mut store, dir, &report)?;
    }
    Ok(())
}

/// Confirm an interrupted coordinator is quiescent without launching or killing
/// any agent. A missing spawn record is deliberately not interpreted as absence.
pub fn recover(dir: &Path) -> Result<Report, String> {
    recover_interrupted(dir, false)
}

mod interrupted;
pub(super) use interrupted::recover_interrupted;

pub async fn resume(dir: &Path, timeout: u64) -> Result<Report, String> {
    let baseline = stop_digest(dir)?;
    resume_reviewed_stop(dir, timeout, &baseline).await
}

pub fn stop_digest(dir: &Path) -> Result<String, String> {
    let path = dir.join("stop.request");
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("absent".into()),
        Err(e) => Err(e.to_string()),
        Ok(metadata) if metadata.is_file() && metadata.len() <= 20 * 1024 => {
            Ok(digest(&fs::read(path).map_err(|e| e.to_string())?))
        }
        Ok(_) => Err("invalid stop marker".into()),
    }
}

pub async fn resume_reviewed_stop(
    dir: &Path,
    timeout: u64,
    baseline: &str,
) -> Result<Report, String> {
    if baseline != "absent"
        && (baseline.len() != 64 || !baseline.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err("invalid reviewed stop digest".into());
    }
    if timeout == 0 {
        return Err("timeout must be positive".into());
    }
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if report.state != "stopped" {
        return Err("explicit resume requires a confirmed stopped run".into());
    }
    editable(&report)?;
    let mut repository_lease = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    validate_checkpoints(&report, dir)?;
    report.plan.validate()?;
    validate_permissions(&report.plan)?;
    let git = Repository::open(&report.repo)?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err("target Git state changed before resume".into());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Explicit resume revoked prior integration approval",
    )?;
    store.record(
        "resume",
        &serde_json::json!({"requested_at_ms":now()}),
        "Human resumed confirmed stopped attempts",
    )?;
    for task in &mut report.tasks {
        if (task.status == "stopped" || task.status == "blocked") && !tool_rejected(dir, task)? {
            if let Some(context) = crate::cooperative::prepare_continuation(
                &mut store,
                &task.id,
                now()
                    .try_into()
                    .map_err(|_| "continuation time exceeds supported range")?,
            )? {
                task.checkpoint_context = Some(context);
            }
            task.status = "pending".into();
        }
    }
    report.tests.clear();
    report.candidate = None;
    report.diff_hash = None;
    report.error = None;
    let marker = dir.join("stop.request");
    {
        let _stop_lease = crate::leases::Lease::acquire(&dir.join("stop.lock"))?;
        if marker.exists() && stop_digest(dir)? == baseline {
            fs::remove_file(marker).map_err(|e| e.to_string())?;
        }
    }
    drive(report, dir, &mut store, timeout, &mut repository_lease).await
}
