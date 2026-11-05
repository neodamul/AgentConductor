use super::*;

/// Derives evidence only; this function cannot grant or consume permission.
pub fn tool_context(
    dir: &Path,
    id: &str,
    request: &crate::gates::ToolRequest,
) -> Result<crate::gates::Context, String> {
    let report = load(dir)?;
    report.plan.validate()?;
    let index = task_index(&report, id)?;
    let task = &report.tasks[index];
    let session = task
        .session_id
        .as_deref()
        .filter(|id| !id.is_empty() && id.len() <= 256)
        .ok_or("native session identity is missing")?;
    let policy = crate::guard::Policy::new(&task.cwd, &report.plan.tasks[index].paths)?;
    if policy.classify(&request.tool, &request.input)? != crate::guard::Disposition::Human {
        return Err("context verification is only for supported human-gated tools".into());
    }
    let target = Repository::open(&report.repo)?;
    if target.head()? != report.base || target.branch()? != report.branch {
        return Err("target Git baseline changed before tool approval".into());
    }
    let workspace = Repository::open(&task.cwd)?;
    if workspace.head()? != task.base {
        return Err("task Git baseline changed before tool approval".into());
    }
    let common = |git: &Repository| -> Result<PathBuf, String> {
        PathBuf::from(git.text(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?)
            .canonicalize()
            .map_err(|e| e.to_string())
    };
    if common(&workspace)? != common(&target)? {
        return Err("task workspace does not belong to the target repository".into());
    }
    // Inspect ignored sensitive targets without reading their contents or adding them to Git.
    let file_identity = match policy.file_target(&request.tool, &request.input)? {
        Some(path) => match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if !metadata.is_file() || metadata.nlink() != 1 {
                        return Err("tool target identity is unsupported".into());
                    }
                    serde_json::json!({"dev":metadata.dev(),"ino":metadata.ino(),"len":metadata.len(),"mode":metadata.mode(),"uid":metadata.uid(),"gid":metadata.gid(),"mtime":metadata.mtime(),"mtime_ns":metadata.mtime_nsec(),"ctime":metadata.ctime(),"ctime_ns":metadata.ctime_nsec()})
                }
                #[cfg(windows)]
                {
                    let id = crate::guard::windows_identity(&path)?;
                    if !metadata.is_file() || id.reparse || id.links != 1 {
                        return Err("tool target identity is unsupported".into());
                    }
                    serde_json::json!({"volume":id.volume,"index":id.index,"len":id.size,"attributes":id.attributes,"created":id.created,"written":id.written})
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && request.tool == "Write" => {
                serde_json::json!({"missing":true})
            }
            Err(e) => return Err(format!("tool target cannot be inspected: {e}")),
        },
        None => serde_json::Value::Null,
    };
    let temporary = dir.join(format!(
        "tool-context-{}-{}.index",
        std::process::id(),
        now()
    ));
    let checkpoint = workspace_checkpoint(&task.cwd, &temporary)?;
    Ok(crate::gates::Context {
        epoch: task.approval_epoch,
        task: task.id.clone(),
        session: session.into(),
        workspace: checkpoint.cwd.to_str().ok_or("non-UTF8 workspace")?.into(),
        plan_sha256: digest(&serde_json::to_vec(&report.plan).map_err(|e| e.to_string())?),
        workspace_sha256: digest(
            &serde_json::to_vec(&(checkpoint, file_identity)).map_err(|e| e.to_string())?,
        ),
    })
}

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
