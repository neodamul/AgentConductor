//! Workspace auto-resume defaults on; manually stopped runs stay stopped.
use super::*;
pub(super) fn enabled() -> bool {
    true
}
impl Startup {
    pub(crate) fn auto_resume(&self) -> bool {
        self.saved
            .lock()
            .map(|s| s.config.auto_resume)
            .unwrap_or(false)
    }
    pub(super) fn recover_last_run(&self) -> Result<(), String> {
        let (run, enabled) = {
            let saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
            (self.last_run(&saved), saved.config.auto_resume)
        };
        let Some(dir) = run else {
            return Ok(());
        };
        if self.ensure_no_live_jobs().is_err() {
            return Ok(());
        }
        let controller = Controller::new(
            dir,
            self.broker.exe.clone(),
            self.events.clone(),
            crate::ui::events::UiSurface::Workspace,
        );
        controller.automatic(enabled)
    }
}
