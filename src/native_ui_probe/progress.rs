use std::time::{Duration, Instant};

const STALE_AFTER: Duration = Duration::from_secs(5);

#[derive(Default)]
pub(super) struct ActiveWork {
    active: Option<TrackedWork>,
}

struct TrackedWork {
    kind: String,
    id: String,
    clock: WaitClock,
}

impl ActiveWork {
    pub(super) fn begin(&mut self, kind: &str, id: &str, now: Instant) {
        if !tracked_kind(kind) {
            return;
        }
        self.active = Some(TrackedWork {
            kind: kind.to_owned(),
            id: id.to_owned(),
            clock: WaitClock::start(now),
        });
    }

    pub(super) fn observe_snapshot(
        &mut self,
        snapshot: &serde_json::Value,
        now: Instant,
        epoch_now_ms: u64,
    ) {
        if snapshot["revision"].as_str().is_none() {
            return;
        }
        let job = &snapshot["job"];
        let (Some(kind), Some(id)) = (job["kind"].as_str(), job["id"].as_str()) else {
            self.active = None;
            return;
        };
        if job["state"] != "running" || !tracked_kind(kind) {
            self.active = None;
            return;
        }
        if let Some(active) = self
            .active
            .as_mut()
            .filter(|active| active.id == id && active.kind == kind)
        {
            active.clock.observe(now);
        } else {
            let started_ms = job["started_ms"].as_u64();
            self.active = Some(TrackedWork {
                kind: kind.to_owned(),
                id: id.to_owned(),
                clock: WaitClock::restore(started_ms, epoch_now_ms, now),
            });
        }
    }

    pub(super) fn observe_current_snapshot(&mut self, snapshot: &serde_json::Value) {
        let epoch_now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u64::MAX.into()) as u64;
        self.observe_snapshot(snapshot, Instant::now(), epoch_now_ms);
    }

    pub(super) fn clear_kind(&mut self, kind: &str) {
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.kind == kind)
        {
            self.active = None;
        }
    }

    pub(super) fn label(&self, kind: &str, language: &str, now: Instant) -> Option<String> {
        self.active
            .as_ref()
            .filter(|active| active.kind == kind)
            .map(|active| label(active.clock, language, now))
    }

    pub(super) fn is_active(&self) -> bool {
        self.active.is_some()
    }
}

fn tracked_kind(kind: &str) -> bool {
    matches!(kind, "chat" | "optimize" | "mcp")
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WaitClock {
    started_at: Instant,
    observed_at: Instant,
}

impl WaitClock {
    pub(super) fn start(now: Instant) -> Self {
        Self {
            started_at: now,
            observed_at: now,
        }
    }

    pub(super) fn restore(started_ms: Option<u64>, epoch_now_ms: u64, now: Instant) -> Self {
        let age = started_ms
            .map(|start| Duration::from_millis(epoch_now_ms.saturating_sub(start)))
            .unwrap_or_default();
        Self {
            started_at: now.checked_sub(age).unwrap_or(now),
            observed_at: now,
        }
    }

    pub(super) fn observe(&mut self, now: Instant) {
        self.observed_at = now;
    }

    pub(super) fn elapsed_at(self, now: Instant) -> Duration {
        now.saturating_duration_since(self.started_at)
    }

    pub(super) fn silent_for_at(self, now: Instant) -> Option<Duration> {
        let silent = now.saturating_duration_since(self.observed_at);
        (silent >= STALE_AFTER).then_some(silent)
    }
}

pub(super) fn format_elapsed(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub(super) fn label(clock: WaitClock, language: &str, now: Instant) -> String {
    let elapsed = format_elapsed(clock.elapsed_at(now));
    let mut label = crate::ui::messages::interpolate(
        crate::ui::messages::in_language(language, "native.elapsed_for"),
        &[("value1", &elapsed)],
    );
    if let Some(silent) = clock.silent_for_at(now) {
        let age = format_elapsed(silent);
        label.push_str(" · ");
        label.push_str(&crate::ui::messages::interpolate(
            crate::ui::messages::in_language(language, "native.no_new_state_for"),
            &[("value1", &age)],
        ));
    }
    label
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/progress.rs"]
mod tests;
