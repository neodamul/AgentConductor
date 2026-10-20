use eframe::egui;
use serde_json::Value;
use std::time::{Duration, Instant};

pub fn room_refresh_interval(snapshot: &Value) -> Duration {
    let state = snapshot["report"]["state"]
        .as_str()
        .or_else(|| snapshot["state"].as_str());
    if snapshot["job"]["state"] == "running"
        || super::super::workspace::profile_tools::metadata_running(snapshot)
        || matches!(state, Some("running" | "ready" | "waiting_availability"))
    {
        Duration::from_secs(1)
    } else {
        Duration::from_secs(5)
    }
}

pub fn refresh_due(
    context: &egui::Context,
    snapshot: Option<&Value>,
    pending: bool,
    next: &mut Option<Instant>,
) -> bool {
    if pending {
        return false;
    }
    let interval = room_refresh_interval(snapshot.unwrap_or(&Value::Null));
    let now = Instant::now();
    let deadline = next.get_or_insert(now + interval);
    *deadline = (*deadline).min(now + interval);
    if now >= *deadline {
        *deadline = now + interval;
        context.request_repaint_after(interval);
        true
    } else {
        context.request_repaint_after(*deadline - now);
        false
    }
}
