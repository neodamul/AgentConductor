use eframe::egui::{self, Color32};
use serde_json::Value;
use std::time::{Duration, Instant};

pub fn show(
    snapshot: Option<&Value>,
    pending: bool,
    error: Option<&str>,
    ui: &mut egui::Ui,
) -> bool {
    ui.heading("Agent Room");
    let refresh = ui
        .add_enabled(!pending, egui::Button::new("↻ Refresh"))
        .accessible_name("Refresh Agent Room")
        .clicked();
    if let Some(snapshot) = snapshot {
        super::render::snapshot(snapshot, ui);
    } else {
        ui.label("Connecting to the Rust run controller…");
    }
    if pending {
        ui.spinner();
    }
    if let Some(error) = error {
        ui.colored_label(Color32::DARK_RED, error);
    }
    refresh
}

pub fn room_refresh_interval(snapshot: &Value) -> Duration {
    let state = snapshot["report"]["state"]
        .as_str()
        .or_else(|| snapshot["state"].as_str());
    if snapshot["job"]["state"] == "running"
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
