use eframe::egui::Color32;
use serde_json::Value;

pub fn planning_status(snapshot: &Value, now_ms: u64) -> Option<(String, u64, u64)> {
    let job = &snapshot["job"];
    if job["kind"] != "plan" || job["state"] != "running" {
        return None;
    }
    let started = job["started_ms"].as_u64()?;
    Some((
        snapshot["config"]["coordinator"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
        now_ms.saturating_sub(started) / 1000,
        job["output"]["bytes"].as_u64().unwrap_or_default(),
    ))
}

pub fn planning_activity_color(snapshot: &Value, now_ms: u64, animate: bool) -> Option<Color32> {
    planning_status(snapshot, now_ms)?;
    Some(super::super::render::status_color_with_motion(
        "running",
        now_ms as f64 / 1000.0,
        animate,
    ))
}
