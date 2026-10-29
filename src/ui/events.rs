use eframe::egui;
pub(super) mod bridge;
pub(super) mod recovery;
#[cfg(test)]
use recovery::RecoveryObserver;
mod hub;
pub use hub::EventHub;
use hub::EventPublisher;
use std::{
    net::{SocketAddr, UdpSocket},
    sync::{OnceLock, mpsc},
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

/// Notify a supervising native UI that this CLI child has finished its command.
pub fn notify_external_from_environment() {
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
