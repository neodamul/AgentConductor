use super::*;

impl Startup {
    pub(super) fn launch(
        &self,
        saved: &mut Saved,
        operation: &str,
        args: Vec<String>,
    ) -> Result<serde_json::Value, String> {
        let revision = self.revision(saved)?;
        self.launch_with_revision(saved, operation, args, &revision)
    }

    pub(super) fn launch_with_revision(
        &self,
        saved: &mut Saved,
        operation: &str,
        args: Vec<String>,
        revision: &str,
    ) -> Result<serde_json::Value, String> {
        let mut slot = self
            .broker
            .job
            .lock()
            .map_err(|_| messages::text("native.startup_job_state_poisoned"))?;
        if slot.as_ref().is_some_and(|job| !job.finished) {
            return Err(messages::text("native.startup_operation_still_running").into());
        }
        let result = self
            .broker
            .launch_locked(operation, &args, revision, &mut slot)?;
        saved.job = Some(StoredJob {
            id: result["id"]
                .as_str()
                .ok_or(messages::text("native.missing_job_id"))?
                .into(),
            operation: operation.into(),
            started_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| messages::text("native.clock_out_of_range"))?,
        });
        self.store(saved)?;
        Ok(result)
    }
}
