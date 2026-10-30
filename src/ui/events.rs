use std::sync::{Mutex, mpsc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    StateChanged,
}

#[derive(Default)]
pub struct EventBus {
    subscribers: Mutex<Subscribers>,
}

#[derive(Default)]
struct Subscribers {
    next_id: u64,
    listeners: Vec<(u64, mpsc::SyncSender<Event>, Wake)>,
}

type Wake = std::sync::Arc<dyn Fn() + Send + Sync>;

pub struct Subscription {
    pub id: u64,
    pub events: mpsc::Receiver<Event>,
}

impl EventBus {
    #[cfg(test)]
    pub fn subscribe(&self) -> Result<Subscription, String> {
        self.subscribe_with_wakeup(|| {})
    }

    pub fn subscribe_with_wakeup(
        &self,
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
            .push((id, sender, std::sync::Arc::new(wake)));
        Ok(Subscription { id, events })
    }

    pub fn publish(&self, source: Option<u64>, event: Event) {
        let listeners = match self.subscribers.lock() {
            Ok(subscribers) => subscribers.listeners.clone(),
            Err(_) => return,
        };
        let mut disconnected = Vec::new();
        for (id, sender, wake) in listeners {
            if Some(id) == source {
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
                .retain(|(id, _, _)| !disconnected.contains(id));
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
        | "messenger" => None,
        _ => Some(Event::StateChanged),
    }
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/events.rs"]
mod tests;
