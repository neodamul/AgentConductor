use super::*;
impl Room {
    pub(super) fn dispatch_run(&self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        let controller = self
            .controller
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?
            .clone();
        let Some(controller) = controller else {
            return Err(crate::ui::messages::text("native.no_active_surface").into());
        };
        let request = serde_json::from_slice::<Request>(bytes).map_err(|e| e.to_string())?;
        if let Request::Briefing {} = &request {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();
            return crate::briefing::collect(
                &controller.dir,
                u64::try_from(now)
                    .map_err(|_| crate::ui::messages::text("native.briefing_clock_overflow"))?,
            );
        }
        if let Request::UndoAmendment { id } = &request {
            workflow::amendment_undo(&controller.dir, id)?;
            return Ok(serde_json::json!({"undone":id}));
        }
        if let Request::ScopeFailure {} = &request {
            return workflow::scope_failure(&controller.dir);
        }
        if let Request::ScopePreview {
            task,
            paths,
            test_command,
        } = &request
        {
            return workflow::recovery_preview(&controller.dir, task, paths, test_command.clone());
        }
        if let Some(startup) = self.surface()? {
            startup.poll_job()?;
        }
        match request {
            Request::Briefing {} => unreachable!("briefing returned before polling mutations"),
            Request::ScopeFailure {}
            | Request::ScopePreview { .. }
            | Request::UndoAmendment { .. } => {
                unreachable!("scope review returned before polling mutations")
            }
            Request::Snapshot {
                icons,
                quota_refresh_last,
            } => controller.snapshot(icons, &quota_refresh_last),
            Request::Evidence {} => controller.evidence(),
            Request::Task { index } => controller.task(index),
            Request::Coordinator {} => controller.coordinator(),
            Request::Action { revision, action } => {
                controller.action(ActionRequest { revision, action })
            }
        }
    }
}
