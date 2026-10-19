use super::*;

pub async fn resume_due(dir: &Path, current_time: i64, timeout: u64) -> Result<Report, String> {
    resume_inner(dir, current_time, timeout, false).await
}

pub async fn resume_availability_due(
    dir: &Path,
    current_time: i64,
    timeout: u64,
) -> Result<Report, String> {
    resume_inner(dir, current_time, timeout, true).await
}

async fn resume_inner(
    dir: &Path,
    current_time: i64,
    timeout: u64,
    automatic: bool,
) -> Result<Report, String> {
    let _lock = Lock::acquire(dir)?;
    let mut report = load(dir)?;
    if automatic
        && (dir.join("stop.request").exists()
            || report.state != "waiting_availability"
            || !super::availability_timer(&report, current_time)
                .iter()
                .any(|timer| timer.at <= current_time))
    {
        return Ok(report);
    }
    if matches!(
        report.state.as_str(),
        "completed" | "waiting_approval" | "approved" | "rejected" | "stopped"
    ) {
        return Ok(report);
    }
    editable(&report)?;
    let mut lease = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
    validate_checkpoints(&report, dir)?;
    report.plan.validate()?;
    validate_permissions(&report.plan)?;
    let git = Repository::open(&report.repo)?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err("target Git state changed before resume".into());
    }
    let mut store = Store::open(dir.join("state.sqlite"))?;
    super::availability_timer::sync_availability(&mut report, dir, &mut store, current_time)?;
    super::assignment::apply_fallbacks_at(&mut report, dir, &mut store, current_time)?;
    report
        .account_restrictions
        .retain(|_, restriction| restriction.available_at.is_none_or(|at| at > current_time));
    for task in &mut report.tasks {
        if task.status == "waiting_availability"
            && task
                .restriction
                .as_ref()
                .is_some_and(|r| r.available_at.is_some_and(|at| at <= current_time))
            && task.retry_not_before <= current_time
            && task.automatic_retries < 3
        {
            task.status = "pending".into();
            task.restriction = None;
            task.automatic_retries += 1;
        }
    }
    // Explicit Resume is also the human retry path for blocked attempts.
    for task in &mut report.tasks {
        if task.status == "blocked" && !tool_rejected(dir, task)? {
            task.status = "pending".into();
        }
    }
    if !report.tasks.iter().any(|task| task.status == "pending") {
        if report
            .tasks
            .iter()
            .any(|task| task.status == "waiting_availability")
        {
            report.state = "waiting_availability".into();
        }
        save(&mut store, dir, &report)?;
        return Ok(report);
    }
    store.record(
        "resume",
        &serde_json::json!({"at":current_time}),
        "Coordinator claimed due sessions under exclusive run lock",
    )?;
    report.tests.clear();
    report.candidate = None;
    report.diff_hash = None;
    report.error = None;
    drive(report, dir, &mut store, timeout, &mut lease).await
}
