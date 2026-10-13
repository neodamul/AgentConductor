//! DD-135: update requests from the page. Installing ends this process, and with it every
//! worker the app started, so it is refused while any run or app job is active.
use super::*;

impl Room {
    pub(super) fn update(&self, kind: &str) -> Result<serde_json::Value, String> {
        match kind {
            "check_update" => crate::update::check_async(),
            "apply_update" => {
                self.ensure_update_idle()?;
                crate::update::apply_async()?;
            }
            _ => {}
        }
        Ok(crate::update::status())
    }

    fn ensure_update_idle(&self) -> Result<(), String> {
        let busy = || crate::ui::messages::text("native.finish_or_stop_runs_before_updating");
        if let Some(startup) = self.surface()? {
            startup.ensure_idle().map_err(|_| busy())?;
        }
        if let Some(controller) = self
            .controller
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?
            .clone()
            && controller.busy()
        {
            return Err(busy().into());
        }
        // Runs of every saved workspace, not only the open one.
        for workspace in workspaces::saved()? {
            let Ok(entries) = fs::read_dir(&workspace) else {
                continue;
            };
            for run in entries.flatten().map(|e| e.path()) {
                if !run.join("report.json").is_file() {
                    continue;
                }
                if let Ok(report) = workflow::load(&run)
                    && (report.state == "running"
                        || report.tasks.iter().any(|t| t.status == "running"))
                {
                    return Err(busy().into());
                }
            }
        }
        Ok(())
    }
}
