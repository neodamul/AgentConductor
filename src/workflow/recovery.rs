use super::*;

pub async fn resume_due(dir: &Path, current_time: i64, timeout: u64) -> Result<Report, String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if matches!(
        report.state.as_str(),
        "completed" | "waiting_approval" | "approved" | "rejected" | "stopped"
    ) {
        return Ok(report);
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
    report.account_restrictions.retain(|_, restriction| {
        restriction
            .available_at
            .is_none_or(|time| time > current_time)
    });
    for task in &mut report.tasks {
        if task.status == "waiting_availability"
            && task
                .restriction
                .as_ref()
                .is_some_and(|r| r.available_at.is_some_and(|t| t <= current_time))
            && task.retry_not_before <= current_time
            && task.automatic_retries < 3
        {
            task.status = "pending".into();
            task.restriction = None;
            task.automatic_retries += 1;
        }
    }
    if !report.tasks.iter().any(|t| t.status == "pending") {
        return Ok(report);
    }
    for task in &mut report.tasks {
        if task.status == "blocked" && !tool_rejected(dir, task)? {
            task.status = "pending".into();
        }
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "resume",
        &serde_json::json!({"at":current_time}),
        "Coordinator claimed due sessions under exclusive run lock",
    )?;
    report.tests.clear();
    report.candidate = None;
    report.diff_hash = None;
    report.error = None;
    drive(report, dir, &mut store, timeout, &mut repository_lease).await
}

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
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    let _repository_lease = crate::leases::Lease::repository(&report.repo)?;
    if report.state != "running" {
        return Err("recovery requires an interrupted running coordinator".into());
    }
    let verify = |path: &Path| -> Result<(), String> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|e| format!("missing process ownership evidence {}: {e}", path.display()))?;
        if !metadata.is_file() || metadata.len() > 8192 {
            return Err("invalid process ownership evidence".into());
        }
        let owner: process::Ownership =
            serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
                .map_err(|e| format!("invalid process ownership evidence: {e}"))?;
        owner.ensure_quiescent()
    };
    for task in report.tasks.iter().filter(|t| t.status == "running") {
        verify(&task.artifacts.join("process/owner.json"))?;
    }
    // Earlier attempts and integration tests may also have outlived a crash.
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("task-")
            && !name.starts_with("test-")
            && !name.starts_with("review-")
            && !name.starts_with("coordinator-ui-")
        {
            continue;
        }
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            return Err("invalid attempt directory in recovery inventory".into());
        }
        let logs = if name.starts_with("test-") {
            entry.path()
        } else {
            entry.path().join("process")
        };
        if logs.exists() {
            if !fs::symlink_metadata(&logs)
                .map_err(|e| e.to_string())?
                .is_dir()
            {
                return Err("invalid process log directory in recovery inventory".into());
            }
            verify(&logs.join("owner.json"))?;
        }
    }
    for (index, task) in report.tasks.iter_mut().enumerate() {
        if !matches!(task.status.as_str(), "running" | "pending")
            || !task.artifacts.join("process").exists()
        {
            continue;
        }
        let mut file = fs::File::open(task.artifacts.join("process/stdout.jsonl"))
            .map_err(|e| e.to_string())?;
        let profile = report
            .plan
            .agents
            .iter()
            .find(|a| a.id == report.plan.tasks[index].agent)
            .ok_or("missing task profile")?;
        let mut decoder = Decoder::new(profile.parser, 1024 * 1024);
        let mut buffer = [0; 8192];
        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            let Ok(batch) = decoder.push(&buffer[..n]) else {
                break;
            };
            for event in batch {
                if let Some(id) = event.session_id {
                    if profile.permission_mode == "native_callback"
                        || profile.cooperative == crate::plan::CooperativeMode::CheckpointV1
                    {
                        if task.session_id.as_deref() == Some(id.as_str()) {
                            task.session_established = true;
                        }
                    } else {
                        task.session_id = Some(id);
                    }
                }
            }
        }
        task.status = "stopped".into();
        task.attempt = task.attempt.max(1);
        task.finished_ms = Some(now());
        task.error = Some(
            "Coordinator interrupted; process absence verified; explicit resume required".into(),
        );
    }
    report.state = "stopped".into();
    report.candidate = None;
    report.diff_hash = None;
    capture_checkpoints(&mut report, dir)?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record("recovery", &serde_json::json!({"verified_at_ms":now()}), "Recovered interrupted coordinator only after verifying every recorded process and group absent")?;
    store.record(
        "approval",
        &serde_json::Value::Null,
        "Crash recovery revoked candidate approval",
    )?;
    save(&mut store, dir, &report)?;
    Ok(report)
}

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
