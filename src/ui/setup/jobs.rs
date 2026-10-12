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
