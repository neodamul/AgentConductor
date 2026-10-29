use super::State;
use serde_json::Value;
use std::time::{Duration, Instant};

const FOLLOW_UP_INTERVAL: Duration = Duration::from_secs(1);
const MAX_FOLLOW_UPS: u8 = 20;

pub struct ToolRefreshFollowUp {
    id: String,
    remaining: u8,
    next: Option<Instant>,
}

impl ToolRefreshFollowUp {
    pub fn new(id: String) -> Self {
        Self {
            id,
            remaining: MAX_FOLLOW_UPS,
            next: None,
        }
    }

    pub fn due(&mut self, now: Instant) -> bool {
        if self.next.is_some_and(|deadline| now >= deadline) {
            self.next = None;
            true
        } else {
            false
        }
    }

    pub fn observe(&mut self, snapshot: &Value, now: Instant) -> bool {
        let still_pending = snapshot["tool_requests"]
            .as_array()
            .is_some_and(|requests| {
                requests
                    .iter()
                    .any(|request| request["id"] == self.id && request["status"] == "pending")
            });
        if !still_pending || self.remaining == 0 {
            self.next = None;
            return false;
        }
        self.remaining -= 1;
        self.next = Some(now + FOLLOW_UP_INTERVAL);
        true
    }
}

pub fn observe_snapshot(state: &mut State, snapshot: &Value, now: Instant) {
    let Some(follow_up) = state.tool_refresh_follow_up.as_mut() else {
        return;
    };
    if !follow_up.observe(snapshot, now) {
        state.tool_refresh_follow_up = None;
    }
}

pub fn refresh_due(state: &mut State, now: Instant) -> bool {
    state
        .tool_refresh_follow_up
        .as_mut()
        .is_some_and(|follow_up| follow_up.due(now))
}

pub fn next_deadline(state: &State) -> Option<Instant> {
    state
        .tool_refresh_follow_up
        .as_ref()
        .and_then(|follow_up| follow_up.next)
}
