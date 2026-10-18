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
