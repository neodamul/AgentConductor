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
                .any(|t| t.at <= current_time))
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
    let mut repository_lease = crate::leases::WorkspaceLease::acquire(&report.repo, dir)?;
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
    // GitHub #625 / DD-132: Resume on a failed run retries its failed tasks from the retained work and session
    // and starts the coordinator's repair rounds again (the dependents blocked by them are retried below).
    // A blocked coordinator review that proposed no revisions is retried the same way: the verification and the
    // review start again instead of leaving the run with nothing to do.
    let from_failed = !automatic && (report.state == "failed" || review_blocked(&report));
    if from_failed {
        report.review_round = 0;
        report.review_error = None;
        report.latest_review = None;
        for task in &mut report.tasks {
            if task.status == "failed" {
                task.status = "pending".into();
                task.retry_not_before = 0;
            }
        }
    }
    // Blocked tasks (for example a timed-out session) are retried by Resume.
    for task in &mut report.tasks {
        if matches!(task.status.as_str(), "blocked" | "permission_denied")
            && !tool_rejected(dir, task)?
        {
            task.status = "pending".into();
        }
    }
    // A failed integration verification (every task completed) is run again.
    let reverify = from_failed && report.tasks.iter().all(|t| t.status == "completed");
    if !reverify && !report.tasks.iter().any(|t| t.status == "pending") {
        if report
            .tasks
            .iter()
            .any(|t| t.status == "waiting_availability")
        {
            report.state = "waiting_availability".into();
        } else if let Some(blocked) = report.tasks.iter().find(|task| task.status == "blocked") {
            report.state = "blocked".into();
            report.error = blocked.error.clone();
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
    drive(report, dir, &mut store, timeout, &mut repository_lease).await
}

/// A run whose coordinator review ended `blocked` without any revision to apply.
fn review_blocked(report: &Report) -> bool {
    report.state == "blocked"
        && report.tasks.iter().all(|t| t.status == "completed")
        && report.latest_review.as_ref().is_some_and(|r| {
            r.verdict == crate::coordinator::Verdict::Blocked && r.revisions.is_empty()
        })
}

/// Why Resume has nothing to do: the first task that is not completed as `(task, status)`, or `("", state)` when
/// every task is completed. `None` when Resume can start something.
pub fn resume_blocker(dir: &Path, report: &Report) -> Result<Option<(String, String)>, String> {
    let mut resumable = report.state == "failed" || review_blocked(report);
    for task in &report.tasks {
        resumable |= match task.status.as_str() {
            "pending" | "waiting_availability" => true,
            "blocked" | "permission_denied" => !tool_rejected(dir, task)?,
            _ => false,
        };
    }
    if resumable {
        return Ok(None);
    }
    Ok(Some(
        report
            .tasks
            .iter()
            .find(|t| t.status != "completed")
            .map_or((String::new(), report.state.clone()), |t| {
                (t.id.clone(), t.status.clone())
            }),
    ))
}
