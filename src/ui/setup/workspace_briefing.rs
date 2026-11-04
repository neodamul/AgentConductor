//! #427: the Workspace reuses the read-only run briefing, without opening a controller.
use super::*;
impl Startup {
    pub(super) fn workspace_briefing(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        let run = self
            .active_run_for(saved)?
            .ok_or_else(|| messages::text("native.no_active_surface"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis();
        crate::briefing::collect(
            &run,
            u64::try_from(now).map_err(|_| messages::text("native.briefing_clock_overflow"))?,
        )
    }
}
