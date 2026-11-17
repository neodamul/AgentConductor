//! GitHub #828: an agent's task sessions across runs, and clean-up of finished ones.
use super::*;

/// Marker in a task's artifacts folder: the session was cleaned up and leaves the agent's list.
pub const SESSION_CLEANED: &str = "session-cleaned";

/// Removes a finished task's linked worktree and hides the session; logs stay until cache clean-up.
/// Only a run that is completed, stopped or failed may lose a worktree, so a pending merge keeps its files.
pub fn clean_session(dir: &Path, task_id: &str) -> Result<(), String> {
    let _lock = Lock::acquire(dir)?;
    let report = load(dir)?;
    if !matches!(report.state.as_str(), "completed" | "stopped" | "failed") {
        return Err(crate::ui::messages::text("native.session_run_still_active").into());
    }
    let task = report
        .tasks
        .iter()
        .find(|task| task.id == task_id)
        .ok_or("unknown task session")?;
    // Paths may be recorded through a symlinked temp folder (/var vs /private/var), so compare real paths.
    let real = |p: &Path| p.canonicalize().ok();
    if let (Some(cwd), Some(run)) = (real(&task.cwd), real(dir))
        && Some(&cwd) != real(&report.repo).as_ref()
        && cwd.starts_with(&run)
    {
        crate::git::Repository::open(&report.repo)?.remove_worktrees_under(&cwd)?;
    }
    fs::write(task.artifacts.join(SESSION_CLEANED), b"").map_err(|e| e.to_string())
}

/// GitHub #1056: a completed (merged) run can no longer resume, so its sessions leave the agents'
/// lists. The merge already released the worktrees; logs and results stay viewable.
pub(super) fn mark_completed_sessions(report: &Report) {
    for task in &report.tasks {
        if task.started_ms.is_some() && task.artifacts.is_dir() {
            let _ = fs::write(task.artifacts.join(SESSION_CLEANED), b"");
        }
    }
}

/// GitHub #1056: cleans the sessions of a completed run that are still listed (runs merged before
/// this clean-up existed); other runs are left alone because they can still resume. Returns how many.
pub fn clean_completed_sessions(dir: &Path) -> Result<usize, String> {
    let report = load(dir)?;
    if report.state != "completed" {
        return Ok(0);
    }
    let mut cleaned = 0;
    for task in &report.tasks {
        if task.started_ms.is_none()
            || !task.artifacts.is_dir()
            || task.artifacts.join(SESSION_CLEANED).exists()
        {
            continue;
        }
        clean_session(dir, &task.id)?;
        cleaned += 1;
    }
    Ok(cleaned)
}
