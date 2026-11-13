//! Native run actions; slow operations remain owned child processes.
use super::*;
impl Controller {
    pub(in crate::ui) fn action(
        &self,
        request: ActionRequest,
    ) -> Result<serde_json::Value, String> {
        if let Action::Stop { reason } = &request.action {
            workflow::stop(&self.dir, reason)?;
            return Ok(
                serde_json::json!({"accepted":true,"kind":"stop","message":crate::ui::messages::text("native.stop_requested_inspect_task_status_until_processes_are")}),
            );
        }
        if let Action::ToolApprove { sha } | Action::ToolReject { sha } = &request.action
            && workflow::load(&self.dir)?.state == "running"
        {
            // DD-32: a paused worker's request is decided at once, bound to its exact
            // SHA, without waiting for another UI job (such as the resumed coordinator).
            let allow = matches!(request.action, Action::ToolApprove { .. });
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?
                .block_on(workflow::decide_live_request(
                    &self.dir, sha, allow, 300, &self.exe,
                ))?;
            return Ok(
                serde_json::json!({"accepted":true,"kind":request.action.label(),"decided":true,"message":crate::ui::messages::text("native.decision_recorded_the_paused_worker_continues")}),
            );
        }
        self.poll()?;
        let mut slot = self.lock_job()?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err(
                crate::ui::messages::text("native.another_ui_action_is_still_running").into(),
            );
        }
        let report = workflow::load(&self.dir)?;
        if workflow::revision(&report)? != request.revision {
            return Err(crate::ui::messages::text(
                "native.run_changed_since_review_refresh_before_acting",
            )
            .into());
        }
        let args = request.action.args(&report, &self.dir)?;
        if matches!(request.action, Action::Resume {})
            && let Some(parent) = self.dir.parent()
        {
            let marker = parent.join("phase-chain.stop");
            if marker.exists() {
                fs::remove_file(marker).map_err(|e| e.to_string())?;
            }
        }
        let mut command = vec![
            "ui-action".into(),
            self.dir.to_string_lossy().into_owned(),
            request.revision.clone(),
        ];
        command.extend(args);
        self.launch_locked(
            request.action.label(),
            &command,
            &request.revision,
            &mut slot,
        )
    }
}
