//! R-118 / DD-127: closing the window while a run works asks first. Runs are separate processes
//! (their own process group, no job shared with the app), so they keep working after the app
//! ends; the operator may instead stop them before the window closes.
use super::*;
use std::sync::atomic::Ordering;

impl Room {
    /// Runs whose coordinator holds the run lock, in every saved workspace.
    pub fn active_runs(&self) -> Vec<PathBuf> {
        let language = crate::ui::language::load();
        update::saved_runs()
            .unwrap_or_default()
            .into_iter()
            .filter(|(name, run)| {
                matches!(
                    crate::ui::update_guard::run_blocker(&language, name, run),
                    Some(crate::ui::update_guard::Blocker::Active { .. })
                )
            })
            .map(|(_, run)| run)
            .collect()
    }

    /// The operator confirmed closing; the native shell closes the window after this reply.
    pub fn closing(&self) -> bool {
        self.closing.load(Ordering::Acquire)
    }

    pub(super) fn close_window(&self, stop: bool) -> Result<serde_json::Value, String> {
        let mut stopped = Vec::new();
        if stop {
            for run in self.active_runs() {
                match workflow::stop(&run, "Stopped when the window was closed") {
                    Ok(()) => {}
                    // A working coordinator keeps the request and stops after draining its jobs.
                    Err(e) if e.starts_with("unknown process ownership") => {}
                    Err(e) => return Err(e),
                }
                stopped.push(run);
            }
        }
        self.closing.store(true, Ordering::Release);
        Ok(serde_json::json!({"closing": true, "stopped": stopped}))
    }
}
