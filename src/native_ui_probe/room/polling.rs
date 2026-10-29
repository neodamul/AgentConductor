use eframe::egui;
use serde_json::Value;
use std::time::{Duration, Instant};

pub fn room_refresh_interval(snapshot: &Value) -> Duration {
    let state = snapshot["report"]["state"]
        .as_str()
        .or_else(|| snapshot["state"].as_str());
    if snapshot["job"]["state"] == "running"
        || super::super::workspace::profile_tools::metadata_running(snapshot)
        || matches!(state, Some("running" | "ready"))
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
    let due = refresh_due_at(now, interval, next);
    if due {
        context.request_repaint_after(interval);
    } else if let Some(deadline) = next {
        context.request_repaint_after(deadline.saturating_duration_since(now));
    }
    due
}

pub fn refresh_due_at(now: Instant, interval: Duration, next: &mut Option<Instant>) -> bool {
    let deadline = next.get_or_insert(now + interval);
    *deadline = (*deadline).min(now + interval);
    if now >= *deadline {
        *deadline = now + interval;
        true
    } else {
        false
    }
}
