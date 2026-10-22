//! DD-132 / DD-173: a run whose tasks all completed but whose integration verification failed,
//! or whose task failed, is repaired by the coordinator: it reviews the evidence, revises the tasks that must change,
//! and the run resumes. Three review rounds bound the loop; then a human decides.
use super::*;

/// A failure the coordinator may decide: integration verification failed after every task
/// completed, or a task failed (including one that stalled or exited without a result). Scope
/// escapes, waits for an account and approvals need a person and are never triaged.
pub fn repairable(report: &Report) -> bool {
    let error = report.error.as_deref().unwrap_or_default();
    let all_completed = report.tasks.iter().all(|t| t.status == "completed");
    report.state == "failed"
        && !error.starts_with("out-of-scope change:")
        && !report
            .tasks
            .iter()
            .any(|t| t.status == "waiting_availability")
        && (report.tasks.iter().any(|t| t.status == "failed")
            || (error.starts_with("integration ") && all_completed))
}

/// Repairs `report` while it stays repairable, returning the latest state.
pub async fn repair(dir: &Path, timeout: u64, mut report: Report) -> Result<Report, String> {
    while repairable(&report) {
        if report.review_round >= 3 {
            let _lock = Lock::acquire(dir)?;
            let mut store = Store::open(dir.join("state.sqlite"))?;
            let mut latest = load(dir)?;
            latest.review_error = Some(
                "three coordinator repair rounds did not resolve the failure; revise or reassign a task".into(),
            );
            save(&mut store, dir, &latest)?;
            return Ok(latest);
        }
        // Without a recorded coordinator the failure stays for the operator.
        let Some(coordinator) = crate::native_guard::orchestrator_profile(dir) else {
            return Ok(report);
        };
        // A failed review keeps its error on the report (`review_error`) for the operator.
        match review_as(dir, coordinator, true).await {
            Ok(proposal) if proposal.verdict == crate::coordinator::Verdict::Revise => {}
            _ => return load(dir),
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        report = resume_due(dir, now, timeout).await?;
    }
    Ok(report)
}
