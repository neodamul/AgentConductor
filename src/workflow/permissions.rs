use super::*;

pub async fn tool_decide(dir: &Path, id: &str, allow: bool, lifetime: i64) -> Result<(), String> {
    if load(dir)?.state == "running" {
        return tool_decide_live(dir, id, allow, lifetime).await;
    }
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let _repository = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    validate_checkpoints(&report, dir)?;
    if !(1..=900).contains(&lifetime) {
        return Err("tool approval lifetime must be 1..900 seconds".into());
    }
    let mut ledger = crate::gates::Ledger::open(&dir.join("state.sqlite"))?;
    let receipt = ledger.receipt(id)?.ok_or("unknown tool request SHA-256")?;
    let index = task_index(&report, &receipt.context.task)?;
    if report.tasks[index].tool_request.as_deref() != Some(id) {
        return Err("tool request is not the task's current pending decision".into());
    }
    let context = verify_tool_context(
        dir,
        &receipt.context.task,
        &receipt.request,
        &std::env::current_exe().map_err(|e| e.to_string())?,
    )
    .await?;
    let time = (now() / 1000) as i64;
    ledger.decide(id, id, allow, &context, time, time + lifetime)?;
    let stopped = report.state == "stopped";
    let mut store = Store::open(dir.join("state.sqlite"))?;
    if allow {
        let epoch = report.tasks[index].approval_epoch;
        invalidate(&mut report, index, &mut store)?;
        report.tasks[index].approval_epoch = epoch;
        report.tasks[index].tool_request = Some(id.into());
    } else {
        report.tasks[index].status = "blocked".into();
        report.state = "blocked".into();
        report.candidate = None;
        report.tests.clear();
        report.diff_hash = None;
        store.record(
            "approval",
            &serde_json::Value::Null,
            "Tool rejection does not authorize merge",
        )?;
    }
    if stopped {
        report.state = "stopped".into();
    }
    save(&mut store, dir, &report)
}

/// Decides a request a running worker is paused on. The coordinator owns the run
/// lock and report, so only the transactional ledger changes here.
pub(super) async fn tool_decide_live(
    dir: &Path,
    id: &str,
    allow: bool,
    lifetime: i64,
) -> Result<(), String> {
    let engine = std::env::current_exe().map_err(|e| e.to_string())?;
    decide_live_request(dir, id, allow, lifetime, &engine).await
}

/// Live decision with an explicit context-verifier executable, e.g. from the Agent Room.
pub async fn decide_live_request(
    dir: &Path,
    id: &str,
    allow: bool,
    lifetime: i64,
    engine: &Path,
) -> Result<(), String> {
    if !(1..=900).contains(&lifetime) {
        return Err("tool approval lifetime must be 1..900 seconds".into());
    }
    let report = load(dir)?;
    let mut ledger = crate::gates::Ledger::open(&dir.join("state.sqlite"))?;
    let receipt = ledger.receipt(id)?.ok_or("unknown tool request SHA-256")?;
    let task = &report.tasks[task_index(&report, &receipt.context.task)?];
    if receipt.status != "pending"
        || task.status != "running"
        || task.session_id.as_deref() != Some(receipt.context.session.as_str())
    {
        return Err("tool request is not awaited by the task's running worker".into());
    }
    let context = verify_tool_context(dir, &receipt.context.task, &receipt.request, engine).await?;
    let time = (now() / 1000) as i64;
    let id = if context == receipt.context {
        id.to_string()
    } else if (crate::gates::Context {
        workspace_sha256: receipt.context.workspace_sha256.clone(),
        ..context.clone()
    }) == receipt.context
    {
        // Other writers changed the shared workspace while the exact command waited;
        // bind the decision to the state it is made in.
        let fresh = ledger.request(&context, &receipt.request, time)?;
        ledger.supersede(id)?;
        fresh
    } else {
        return Err("tool request binding changed; decision refused".into());
    };
    ledger.decide(&id, &id, allow, &context, time, time + lifetime)
}

pub(super) fn tool_rejected(dir: &Path, task: &TaskReport) -> Result<bool, String> {
    match task.tool_request.as_deref() {
        Some(id) => Ok(crate::gates::Ledger::open(&dir.join("state.sqlite"))?
            .receipt(id)?
            .is_some_and(|r| r.status == "denied")),
        None => Ok(false),
    }
}

pub fn knowledge_accept(
    dir: &Path,
    lesson: &crate::knowledge::Lesson,
    expected: &str,
) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let report = load(dir)?;
    editable(&report)?;
    let _repository = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    validate_checkpoints(&report, dir)?;
    task_index(&report, &lesson.task)?;
    crate::knowledge::accept(dir, &report.repo, lesson, expected)
}

pub async fn tool_refresh(dir: &Path, id: &str) -> Result<String, String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    editable(&report)?;
    let _repository = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    validate_checkpoints(&report, dir)?;
    let mut ledger = crate::gates::Ledger::open(&dir.join("state.sqlite"))?;
    let receipt = ledger.receipt(id)?.ok_or("unknown tool request SHA-256")?;
    let index = task_index(&report, &receipt.context.task)?;
    if report.tasks[index].tool_request.as_deref() != Some(id)
        || !matches!(receipt.status.as_str(), "pending" | "approved")
    {
        return Err("only the current pending/unconsumed request can be refreshed".into());
    }
    report.tasks[index].approval_epoch = report.tasks[index]
        .approval_epoch
        .checked_add(1)
        .ok_or("approval epoch exhausted")?;
    let mut store = Store::open(dir.join("state.sqlite"))?;
    // Persist revocation binding first: a crash cannot revive the old permission.
    save(&mut store, dir, &report)?;
    let context = verify_tool_context(
        dir,
        &receipt.context.task,
        &receipt.request,
        &std::env::current_exe().map_err(|e| e.to_string())?,
    )
    .await?;
    let fresh = ledger.request(&context, &receipt.request, (now() / 1000) as i64)?;
    ledger.supersede(id)?;
    report.tasks[index].tool_request = Some(fresh.clone());
    if report.state != "stopped" {
        report.tasks[index].status = "waiting_approval".into();
        report.state = "waiting_tool_approval".into();
    }
    save(&mut store, dir, &report)?;
    Ok(fresh)
}
