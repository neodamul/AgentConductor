use serde_json::Value;

pub fn can_archive_after_stop(snapshot: &Value) -> bool {
    snapshot["job"]["state"] != "running"
        && snapshot["metadata_job"]["state"] != "running"
        && snapshot["metadata_queue"]
            .as_array()
            .is_none_or(Vec::is_empty)
}

pub fn request() -> &'static [u8] {
    br#"{"kind":"stop_planning"}"#
}
