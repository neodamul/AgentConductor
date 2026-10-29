use super::*;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn next(profiles: &Value, last: &BTreeMap<String, u64>, now: u64) -> (Option<String>, Option<u64>) {
    let mut delay = None;
    for profile in profiles.as_array().into_iter().flatten() {
        if !matches!(profile["parser"].as_str(), Some("claude" | "codex"))
            && profile["quota_command"].is_null()
        {
            continue;
        }
        let Some(id) = profile["id"].as_str() else {
            continue;
        };
        let minutes = profile["quota_refresh_minutes"]
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .unwrap_or(5);
        let previous = last.get(id).copied();
        if crate::quota::refresh_due(previous, now, minutes) {
            return (Some(id.into()), None);
        }
        let remaining = crate::quota::refresh_interval_ms(minutes) - (now - previous.unwrap());
        delay = Some(delay.map_or(remaining, |current: u64| current.min(remaining)));
    }
    (None, delay)
}

impl Probe {
    fn due_quota(&self, now: u64, minimized: bool) -> (Option<String>, Option<u64>) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return (None, None);
        };
        if minimized
            || self.room_view.is_some()
            || self.pending.is_some()
            || self.poll_pending.is_some()
            || self.starting_run.is_some()
            || self.coordinator_chat.awaiting()
            || snapshot["job"]["state"] == "running"
            || snapshot["run_state"] == "running"
            || workspace::profile_tools::metadata_running(snapshot)
        {
            return (None, None);
        }
        next(
            &snapshot["config"]["profiles"],
            &self.quota_refresh_last,
            now,
        )
    }

    pub(super) fn refresh_quotas(&mut self, context: &eframe::egui::Context, now: u64) {
        if self.worker.is_none() {
            return;
        }
        let minimized = context.input(|input| input.viewport().minimized == Some(true));
        let (agent, delay) = self.due_quota(now, minimized);
        if let Some(agent) = agent {
            self.request_metadata(workspace::profile_tools::Event::Quota(agent.clone()));
            // A rejected enqueue also waits for the next interval instead of spinning.
            self.quota_refresh_last.insert(agent, now);
        } else if let Some(delay) = delay {
            context.request_repaint_after(Duration::from_millis(delay));
        }
    }
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/quota_refresh.rs"]
mod tests;
