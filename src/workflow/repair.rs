//! DD-132: a run whose tasks all completed but whose integration verification failed is
//! repaired by the coordinator: it reviews the evidence, revises the tasks that must change,
//! and the run resumes. Three review rounds bound the loop; then a human decides.
use super::*;

/// Every task completed and integration verification (tests or a blocking report) failed.
pub fn repairable(report: &Report) -> bool {
    report.state == "failed"
        && report
            .error
            .as_deref()
            .is_some_and(|e| e.starts_with("integration "))
        && report.tasks.iter().all(|t| t.status == "completed")
}

/// Repairs `report` while it stays repairable, returning the latest state.
pub async fn repair(dir: &Path, timeout: u64, mut report: Report) -> Result<Report, String> {
    while repairable(&report) {
        if report.review_round >= 3 {
            let _lock = Lock::acquire(dir)?;
            let mut store = Store::open(dir.join("state.sqlite"))?;
            let mut latest = load(dir)?;
            latest.review_error = Some(
                "three coordinator repair rounds did not pass integration verification; revise or reassign a task".into(),
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
