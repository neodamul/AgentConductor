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
