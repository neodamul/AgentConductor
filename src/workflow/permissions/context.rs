use super::*;

pub async fn verify_tool_context(
    dir: &Path,
    id: &str,
    request: &crate::gates::ToolRequest,
    engine: &Path,
) -> Result<crate::gates::Context, String> {
    let report = load(dir)?;
    let task = &report.tasks[task_index(&report, id)?];
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    if dir.starts_with(&report.repo) {
        return Err("tool verification evidence must be outside the repository".into());
    }
    let attempt = dir.join(format!(
        "task-verifier-{}-{}-{}",
        &digest(id.as_bytes())[..16],
        std::process::id(),
        now()
    ));
    fs::create_dir(&attempt).map_err(|e| format!("create verifier attempt: {e}"))?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    store.record(
        "tool_verifier",
        &serde_json::json!({"task":id,"artifacts":attempt}),
        "Owned tool-context verification started; no permission granted",
    )?;
    let input = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    if input.len() > 64 * 1024 {
        return Err("tool verification request exceeds 64 KiB".into());
    }
    let result = process::run_until(
        ProcessSpec {
            executable: engine.into(),
            args: vec![
                "tool-context".into(),
                dir.to_str().ok_or("non-UTF8 run directory")?.into(),
                id.into(),
            ],
            cwd: task.cwd.clone(),
            input,
            log_dir: attempt.join("process"),
            // About a dozen Git processes: on Windows each start costs ~0.3 s plus antivirus
            // scanning, and a busy run exceeded 10 s. Stays inside the hook's 60 s margin.
            timeout: Duration::from_secs(30),
            signal_grace: Duration::from_millis(100),
            environment: Default::default(),
        },
        if report.state == "running" {
            Some(dir.join("stop.request"))
        } else {
            None
        },
    )
    .await?;
    if result.exit_code != Some(0) || result.timed_out || result.stopped {
        return Err(format!(
            "tool context verifier failed; inspect {}",
            attempt.display()
        ));
    }
    let mut bytes = vec![];
    fs::File::open(result.stdout)
        .map_err(|e| e.to_string())?
        .take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 {
        return Err("tool context response exceeds 16 KiB".into());
    }
    let context: crate::gates::Context = serde_json::from_slice(&bytes)
        .map_err(|e| format!("invalid tool context response: {e}"))?;
    let fingerprint = &context.workspace_sha256;
    if context.task != task.id
        || context.epoch != task.approval_epoch
        || Some(context.session.as_str()) != task.session_id.as_deref()
        || Path::new(&context.workspace) != task.cwd.canonicalize().map_err(|e| e.to_string())?
        || context.plan_sha256
            != digest(&serde_json::to_vec(&report.plan).map_err(|e| e.to_string())?)
        || fingerprint.len() != 64
        || !fingerprint
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("verifier response does not match the authoritative task binding".into());
    }
    Ok(context)
}
