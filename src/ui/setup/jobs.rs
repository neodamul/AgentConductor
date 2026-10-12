use super::*;
pub(super) fn state(
    broker: &Controller,
    stored: &Option<StoredJob>,
) -> Result<serde_json::Value, String> {
    let mut current = broker.poll()?;
    let Some(job) = stored else {
        return Ok(current);
    };
    let log = broker
        .dir
        .join(format!("coordinator-ui-{}/process", job.id));
    if !current.is_null() {
        if current["id"] == job.id.as_str() {
            current["started_ms"] = job.started_ms.into();
            current["log"] = serde_json::json!(log);
        }
        return Ok(current);
    }
    let owner: crate::process::Ownership = read(
        &broker
            .dir
            .join(format!("coordinator-ui-{}/process/owner.json", job.id)),
    )?;
    let state = if owner.phase == "terminal" {
        if owner.exit_code == Some(0) {
            "completed"
        } else {
            "failed"
        }
    } else if owner.ensure_quiescent().is_ok() {
        "interrupted"
    } else {
        "running"
    };
    Ok(
        serde_json::json!({"id":job.id,"kind":job.operation,"state":state,"exit_code":owner.exit_code,"termination_signal":owner.termination_signal,"started_ms":job.started_ms,"log":log}),
    )
}

/// DD-91: bytes and last write of the coordinator's own output, so progress is observable.
pub(super) fn activity(log_dir: &Path) -> serde_json::Value {
    let (mut bytes, mut updated) = (0u64, None::<u64>);
    for name in ["stdout.jsonl", "stderr.log"] {
        if let Ok(metadata) = fs::metadata(log_dir.join(name)) {
            bytes += metadata.len();
            let ms = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64);
            updated = updated.max(ms);
        }
    }
    serde_json::json!({"bytes": bytes, "updated_ms": updated})
}

impl Startup {
    pub(super) fn job(&self, s: &Saved) -> Result<serde_json::Value, String> {
        let mut job = jobs::state(&self.broker, &s.job)?;
        if job["kind"] == "plan"
            && job["state"] != "running"
            && s.proposal
                .as_ref()
                .is_some_and(|p| p.directory.with_extension("stop.request").exists())
        {
            job["state"] = "stopped".into();
        }
        Ok(job)
    }
}
