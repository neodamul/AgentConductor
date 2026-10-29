use eframe::egui;
use std::sync::{Arc, Mutex, mpsc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSurface {
    Workspace,
    AgentRoom,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiEvent {
    pub surface: UiSurface,
}

impl UiEvent {
    pub fn invalidated(surface: UiSurface) -> Self {
        Self { surface }
    }

    pub fn applies_to(self, surface: UiSurface) -> bool {
        matches!(self.surface, UiSurface::Both) || self.surface == surface
    }

    pub fn for_request(packet: &[u8]) -> Option<Self> {
        let request: serde_json::Value = serde_json::from_slice(packet).ok()?;
        let kind = request["kind"].as_str()?;
        if matches!(
            kind,
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
        ) {
            return None;
        }
        let surface = if request["room"] == true {
            UiSurface::AgentRoom
        } else {
            UiSurface::Workspace
        };
        Some(Self::invalidated(surface))
    }
}

#[derive(Clone)]
struct Subscriber {
    surface: UiSurface,
    context: egui::Context,
    sender: mpsc::SyncSender<UiEvent>,
}

#[derive(Clone)]
pub struct EventHub {
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
    mutations: Arc<Mutex<()>>,
    capacity: usize,
}

impl EventHub {
    pub fn new(capacity: usize) -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            mutations: Arc::new(Mutex::new(())),
            capacity: capacity.max(1),
        }
    }

    pub fn subscribe(&self, surface: UiSurface, context: egui::Context) -> mpsc::Receiver<UiEvent> {
        let (sender, receiver) = mpsc::sync_channel(self.capacity);
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.push(Subscriber {
                surface,
                context,
                sender,
            });
        }
        receiver
    }

    pub fn publish(&self, event: UiEvent) {
        let Ok(mut subscribers) = self.subscribers.lock() else {
            return;
        };
        subscribers.retain(|subscriber| {
            if !event.applies_to(subscriber.surface) {
                return true;
            }
            match subscriber.sender.try_send(event) {
                Ok(()) => subscriber.context.request_repaint(),
                Err(mpsc::TrySendError::Full(_)) => {}
                Err(mpsc::TrySendError::Disconnected(_)) => return false,
            }
            true
        });
    }
}

impl Default for EventHub {
    fn default() -> Self {
        Self::new(1)
    }
}

impl super::Room {
    pub fn subscribe_events(
        &self,
        surface: UiSurface,
        context: egui::Context,
    ) -> mpsc::Receiver<UiEvent> {
        self.events.subscribe(surface, context)
    }

    pub(crate) fn publish_event(&self, event: UiEvent) {
        self.events.publish(event);
    }

    pub(crate) fn dispatch_ui_request(&self, packet: &[u8]) -> Result<serde_json::Value, String> {
        let event = UiEvent::for_request(packet);
        let result = if event.is_some() {
            let _mutation = self
                .events
                .mutations
                .lock()
                .map_err(|_| "UI mutation gate is poisoned".to_owned())?;
            self.dispatch(packet)?
        } else {
            self.dispatch(packet)?
        };
        if let Some(event) = event {
            self.publish_event(event);
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "../../tests/support/ui_events.rs"]
mod tests;
