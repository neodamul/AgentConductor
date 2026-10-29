use eframe::egui;
use std::{
    net::{SocketAddr, UdpSocket},
    process::Command,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSurface {
    Workspace,
    AgentRoom,
    Both,
}

impl UiSurface {
    fn wire_name(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::AgentRoom => "agent-room",
            Self::Both => "both",
        }
    }

    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "workspace" => Some(Self::Workspace),
            "agent-room" => Some(Self::AgentRoom),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
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
    bridge: Option<Arc<EventBridge>>,
}

struct EventBridge {
    address: SocketAddr,
    token: String,
    stopped: Arc<AtomicBool>,
    shutdown: UdpSocket,
}

impl EventBridge {
    fn new(subscribers: Arc<Mutex<Vec<Subscriber>>>) -> std::io::Result<Self> {
        let listener = UdpSocket::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let shutdown = listener.try_clone()?;
        let token = crate::native_guard::session_id().map_err(std::io::Error::other)?;
        let stopped = Arc::new(AtomicBool::new(false));
        let receiver_stop = Arc::clone(&stopped);
        let receiver_token = token.clone();
        thread::Builder::new()
            .name("native-ui-events".into())
            .spawn(move || {
                let mut packet = [0; 128];
                while !receiver_stop.load(Ordering::Acquire) {
                    let Ok((length, _)) = listener.recv_from(&mut packet) else {
                        break;
                    };
                    let Ok(message) = std::str::from_utf8(&packet[..length]) else {
                        continue;
                    };
                    let Some((token, surface)) = message.split_once(':') else {
                        continue;
                    };
                    if token == receiver_token
                        && let Some(surface) = UiSurface::from_wire(surface)
                    {
                        deliver(&subscribers, UiEvent::invalidated(surface));
                    }
                }
            })?;
        Ok(Self {
            address,
            token,
            stopped,
            shutdown,
        })
    }

    fn apply_to(&self, command: &mut Command, surface: UiSurface) {
        command
            .env("AC_UI_EVENT_ADDRESS", self.address.to_string())
            .env("AC_UI_EVENT_TOKEN", &self.token)
            .env("AC_UI_EVENT_SURFACE", surface.wire_name());
    }
}

impl Drop for EventBridge {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        let _ = self.shutdown.send_to(&[], self.address);
    }
}

impl EventHub {
    pub fn new(capacity: usize) -> Self {
        let subscribers = Arc::new(Mutex::new(Vec::new()));
        let bridge = EventBridge::new(Arc::clone(&subscribers))
            .ok()
            .map(Arc::new);
        Self {
            subscribers,
            mutations: Arc::new(Mutex::new(())),
            capacity: capacity.max(1),
            bridge,
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
        deliver(&self.subscribers, event);
    }

    pub(crate) fn configure_child(&self, command: &mut Command, surface: UiSurface) {
        if let Some(bridge) = &self.bridge {
            bridge.apply_to(command, surface);
        }
    }

    #[cfg(test)]
    fn bridge_credentials(&self) -> Option<(SocketAddr, String)> {
        self.bridge
            .as_ref()
            .map(|bridge| (bridge.address, bridge.token.clone()))
    }
}

fn deliver(subscribers: &Arc<Mutex<Vec<Subscriber>>>, event: UiEvent) {
    let Ok(mut subscribers) = subscribers.lock() else {
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

pub(crate) fn notify_external_from_environment() {
    static SENDER: OnceLock<Option<(UdpSocket, SocketAddr, String, String)>> = OnceLock::new();
    let sender = SENDER.get_or_init(|| {
        let address = std::env::var("AC_UI_EVENT_ADDRESS").ok()?.parse().ok()?;
        let token = std::env::var("AC_UI_EVENT_TOKEN").ok()?;
        let surface = std::env::var("AC_UI_EVENT_SURFACE").ok()?;
        UiSurface::from_wire(&surface)?;
        let socket = UdpSocket::bind("127.0.0.1:0").ok()?;
        socket.set_nonblocking(true).ok()?;
        Some((socket, address, token, surface))
    });
    if let Some((socket, address, token, surface)) = sender {
        let message = format!("{token}:{surface}");
        let _ = socket.send_to(message.as_bytes(), address);
    }
}

#[cfg(test)]
fn notify_external(address: SocketAddr, token: &str, surface: UiSurface) {
    if let Ok(socket) = UdpSocket::bind("127.0.0.1:0") {
        let _ = socket.send_to(
            format!("{token}:{}", surface.wire_name()).as_bytes(),
            address,
        );
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
