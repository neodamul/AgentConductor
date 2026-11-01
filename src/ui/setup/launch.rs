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
        saved.job = Some(stored_job(&result, operation)?);
        self.store(saved)?;
        Ok(result)
    }

    /// GitHub #261 (DD-204): the chat worker in its own slot, so a running plan or start job neither blocks it nor is blocked by it.
    pub(super) fn launch_chat_worker(
        &self,
        saved: &mut Saved,
        args: Vec<String>,
        revision: &str,
    ) -> Result<serde_json::Value, String> {
        fs::create_dir_all(&self.chat_broker.dir).map_err(|e| e.to_string())?;
        let mut slot = self
            .chat_broker
            .job
            .lock()
            .map_err(|_| messages::text("native.startup_job_state_poisoned"))?;
        if slot.as_ref().is_some_and(|job| !job.finished) {
            return Err(messages::text("native.chat_still_answering").into());
        }
        let result = self
            .chat_broker
            .launch_locked("chat", &args, revision, &mut slot)?;
        saved.chat_job = Some(stored_job(&result, "chat")?);
        self.store(saved)?;
        Ok(result)
    }
}

fn stored_job(result: &serde_json::Value, operation: &str) -> Result<StoredJob, String> {
    Ok(StoredJob {
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
    })
}
