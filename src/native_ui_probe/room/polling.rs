use eframe::egui;
use serde_json::Value;
use std::time::{Duration, Instant};

pub const RECOVERY_INTERVAL: Duration = Duration::from_secs(30);

pub fn room_refresh_interval(_snapshot: &Value) -> Duration {
    RECOVERY_INTERVAL
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
