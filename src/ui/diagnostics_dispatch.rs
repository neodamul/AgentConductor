use super::{Room, diagnostics};

pub(super) fn dispatch(room: &Room, bytes: &[u8]) -> Result<serde_json::Value, String> {
    let kind = diagnostics::kind(bytes);
    if kind == "client_timeout" {
        if let Ok(report) = serde_json::from_slice(bytes) {
            diagnostics::client_timeout(&report);
        }
        return Ok(serde_json::json!({}));
    }
    diagnostics::start(&kind, bytes.len());
    let started = std::time::Instant::now();
    let result = room.route(bytes);
    diagnostics::done(&kind, started.elapsed(), result.as_ref().err());
    result
}
