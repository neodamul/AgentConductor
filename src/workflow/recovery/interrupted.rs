use super::*;

pub(in crate::workflow) fn recover_interrupted(
    dir: &Path,
    automatic: bool,
) -> Result<Report, String> {
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
        // The desktop's Recover action runs as a UI job recorded in this run directory;
        // its own record is not a survivor of the interrupted coordinator.
        if owner.pid == Some(std::process::id())
            && path
                .strip_prefix(dir)
                .ok()
                .and_then(|p| p.components().next())
                .is_some_and(|c| {
                    c.as_os_str()
                        .to_string_lossy()
                        .starts_with("coordinator-ui-")
                })
        {
            return Ok(());
        }
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
        let mut decoder = Decoder::for_profile(profile, 1024 * 1024);
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
        if automatic
            && !report.plan.tasks[index]
                .prompt
                .contains("## Interrupted execution")
        {
            report.plan.tasks[index].prompt.push_str("\n\n## Interrupted execution\nThe previous coordinator stopped unexpectedly. Process absence was verified. Inspect retained changes and logs, reuse the established session when available, and continue the unfinished work without repeating completed work or broadening its scope.");
        }
        task.status = "stopped".into();
        task.attempt = task.attempt.max(1);
        task.finished_ms = Some(now());
        task.error = Some(
            "Coordinator interrupted; process absence verified; explicit resume required".into(),
        );
    }
    // An interrupted coordinator never applied the reviewed handoff (DD-43).
    for task in report.tasks.iter_mut().filter(|t| t.status == "handoff") {
        task.status = "stopped".into();
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
