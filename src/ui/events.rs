use std::sync::{Mutex, mpsc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    WorkspaceChanged,
    RunChanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventSurface {
    Workspace,
    AgentRoom,
}

impl Event {
    fn targets(self, surface: EventSurface) -> bool {
        matches!(self, Self::RunChanged)
            || matches!(
                (self, surface),
                (Self::WorkspaceChanged, EventSurface::Workspace)
            )
    }
}

#[derive(Default)]
pub struct EventBus {
    subscribers: Mutex<Subscribers>,
}

#[derive(Default)]
struct Subscribers {
    next_id: u64,
    listeners: Vec<(u64, EventSurface, mpsc::SyncSender<Event>, Wake)>,
}

type Wake = std::sync::Arc<dyn Fn() + Send + Sync>;

pub struct Subscription {
    pub id: u64,
    pub events: mpsc::Receiver<Event>,
}

impl EventBus {
    #[cfg(test)]
    pub fn subscribe(&self, surface: EventSurface) -> Result<Subscription, String> {
        self.subscribe_with_wakeup(surface, || {})
    }

    pub fn subscribe_with_wakeup(
        &self,
        surface: EventSurface,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Subscription, String> {
        let mut subscribers = self
            .subscribers
            .lock()
            .map_err(|_| "UI event subscriber list is poisoned".to_owned())?;
        subscribers.next_id = subscribers.next_id.wrapping_add(1).max(1);
        let id = subscribers.next_id;
        let (sender, events) = mpsc::sync_channel(1);
        subscribers
            .listeners
            .push((id, surface, sender, std::sync::Arc::new(wake)));
        Ok(Subscription { id, events })
    }

    pub fn publish(&self, source: Option<u64>, event: Event) {
        // Keep the registry lock out of channel sends and UI wake callbacks.
        let listeners = match self.subscribers.lock() {
            Ok(subscribers) => subscribers.listeners.clone(),
            Err(_) => return,
        };
        let mut disconnected = Vec::new();
        for (id, surface, sender, wake) in listeners {
            if Some(id) == source || !event.targets(surface) {
                continue;
            }
            match sender.try_send(event) {
                Ok(()) => wake(),
                Err(mpsc::TrySendError::Full(_)) => {}
                Err(mpsc::TrySendError::Disconnected(_)) => disconnected.push(id),
            }
        }
        if disconnected.is_empty() {
            return;
        }
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers
                .listeners
                .retain(|(id, _, _, _)| !disconnected.contains(id));
        }
    }
}

pub fn event_for_request(packet: &[u8]) -> Option<Event> {
    let request: serde_json::Value = serde_json::from_slice(packet).ok()?;
    let kind = request["kind"].as_str()?;
    match kind {
        "snapshot"
        | "evidence"
        | "task"
        | "coordinator"
        | "scope_failure"
        | "scope_recovery_preview"
        | "workspaces"
        | "saved_workspaces"
        | "signal_transport"
        | "update_status"
        | "manual_document"
        | "mcp_source_suggestions"
        | "preview_settings"
        | "cache_preview"
        | "messenger"
        | "baseline_status"
        | "resolve_executable" => None,
        "start" | "open_run" => Some(Event::RunChanged),
        _ if request["room"] == true => Some(Event::RunChanged),
        _ => Some(Event::WorkspaceChanged),
    }
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/events.rs"]
mod tests;
