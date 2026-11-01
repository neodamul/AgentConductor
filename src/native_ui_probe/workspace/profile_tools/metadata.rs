use serde_json::Value;

pub fn metadata_running(snapshot: &Value) -> bool {
    snapshot["metadata_job"]["state"] == "running"
        || snapshot["metadata_queue"]
            .as_array()
            .is_some_and(|queue| !queue.is_empty())
}

pub fn metadata_active(snapshot: &Value, kind: &str, agent: &str) -> bool {
    let operation = format!("{kind}:{agent}");
    (snapshot["metadata_job"]["kind"] == operation
        && snapshot["metadata_job"]["state"] == "running")
        || snapshot["metadata_queue"].as_array().is_some_and(|queue| {
            queue
                .iter()
                .any(|item| item["kind"] == kind && item["agent"] == agent)
        })
}

pub fn quota_elapsed(snapshot: &Value, agent: &str, now_ms: u64) -> Option<String> {
    let job = &snapshot["metadata_job"];
    if job["state"] != "running" || job["kind"] != format!("quota:{agent}") {
        return None;
    }
    let started = job["started_ms"].as_u64()?;
    let seconds = now_ms.saturating_sub(started) / 1000;
    Some(format!("{:02}:{:02}", seconds / 60, seconds % 60))
}
