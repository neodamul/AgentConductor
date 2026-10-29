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
    listeners: Vec<(u64, mpsc::Sender<Event>, Wake)>,
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
        let (sender, events) = mpsc::channel();
        subscribers
            .listeners
            .push((id, sender, std::sync::Arc::new(wake)));
        Ok(Subscription { id, events })
    }

    pub fn publish(&self, source: Option<u64>, event: Event) {
        let Ok(mut subscribers) = self.subscribers.lock() else {
            return;
        };
        subscribers.listeners.retain(|(id, sender, wake)| {
            if *id == source.unwrap_or_default() {
                return true;
            }
            if sender.send(event).is_err() {
                return false;
            }
            wake();
            true
        });
    }
}

pub fn event_for_request(packet: &[u8]) -> Option<Event> {
    let request: serde_json::Value = serde_json::from_slice(packet).ok()?;
    let kind = request["kind"].as_str()?;
    match kind {
        "save"
        | "prepare"
        | "draft"
        | "read_plan"
        | "next_phase"
        | "start"
        | "new_goal"
        | "select_goal"
        | "save_plan_rules"
        | "commit_baseline"
        | "ignore_os_files"
        | "create_slug"
        | "save_profiles"
        | "export_settings"
        | "import_settings"
        | "save_messenger"
        | "test_messenger"
        | "mcp_save"
        | "mcp_discover"
        | "mcp_preview"
        | "mcp_apply"
        | "chat"
        | "chat_reset"
        | "optimize_goal"
        | "quota"
        | "discover"
        | "catalog"
        | "stop_planning"
        | "cache_clear"
        | "action"
        | "create_workspace"
        | "switch_workspace"
        | "forget_workspace"
        | "delete_workspace"
        | "set_language"
        | "set_theme"
        | "set_signal_transport"
        | "set_update_settings"
        | "apply_update" => Some(Event::StateChanged),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/events.rs"]
mod tests;
