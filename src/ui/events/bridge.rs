use super::{Event, EventBus, EventSurface};
use std::{
    net::{SocketAddr, UdpSocket},
    process::Command,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

const READ_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct EventEndpoint {
    address: SocketAddr,
    token: String,
}

impl EventEndpoint {
    pub fn configure(&self, command: &mut Command, surface: EventSurface) {
        command
            .env("AC_UI_EVENT_ADDRESS", self.address.to_string())
            .env("AC_UI_EVENT_TOKEN", &self.token)
            .env("AC_UI_EVENT_SURFACE", surface.wire_name());
    }
}

pub struct ProcessEventBridge {
    endpoint: EventEndpoint,
    stopped: Arc<AtomicBool>,
    shutdown: UdpSocket,
    thread: Option<JoinHandle<()>>,
}

impl ProcessEventBridge {
    pub fn new(bus: Arc<EventBus>) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("127.0.0.1:0")?;
        socket.set_read_timeout(Some(READ_TIMEOUT))?;
        let address = socket.local_addr()?;
        let shutdown = socket.try_clone()?;
        let endpoint = EventEndpoint {
            address,
            token: crate::native_guard::session_id().map_err(std::io::Error::other)?,
        };
        let stopped = Arc::new(AtomicBool::new(false));
        let thread_stopped = Arc::clone(&stopped);
        let thread_endpoint = endpoint.clone();
        let thread = thread::Builder::new()
            .name("native-ui-process-events".into())
            .spawn(move || receive(socket, bus, thread_endpoint, thread_stopped))?;
        Ok(Self {
            endpoint,
            stopped,
            shutdown,
            thread: Some(thread),
        })
    }

    pub fn endpoint(&self) -> EventEndpoint {
        self.endpoint.clone()
    }

    #[cfg(test)]
    pub(super) fn credentials(&self) -> (SocketAddr, String) {
        (self.endpoint.address, self.endpoint.token.clone())
    }

    #[cfg(test)]
    pub(super) fn send_test_event(address: SocketAddr, token: &str, surface: EventSurface) {
        if let Ok(socket) = UdpSocket::bind("127.0.0.1:0") {
            let message = format!("{token}:{}", surface.wire_name());
            let _ = socket.send_to(message.as_bytes(), address);
        }
    }
}

impl Drop for ProcessEventBridge {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        let _ = self.shutdown.send_to(&[], self.endpoint.address);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn receive(
    socket: UdpSocket,
    bus: Arc<EventBus>,
    endpoint: EventEndpoint,
    stopped: Arc<AtomicBool>,
) {
    let mut packet = [0; 128];
    while !stopped.load(Ordering::Acquire) {
        let Ok((length, _)) = socket.recv_from(&mut packet) else {
            continue;
        };
        let Ok(message) = std::str::from_utf8(&packet[..length]) else {
            continue;
        };
        if let Some((token, surface)) = message.split_once(':')
            && token == endpoint.token
            && let Some(surface) = EventSurface::from_wire(surface)
        {
            let event = match surface {
                EventSurface::Workspace => Event::WorkspaceChanged,
                EventSurface::AgentRoom => Event::RunChanged,
            };
            bus.publish(None, event);
        }
    }
}

pub(super) fn notify_from_environment() {
    static SENDER: OnceLock<Option<(UdpSocket, SocketAddr, String, EventSurface)>> =
        OnceLock::new();
    let sender = SENDER.get_or_init(|| {
        let address = std::env::var("AC_UI_EVENT_ADDRESS").ok()?.parse().ok()?;
        let token = std::env::var("AC_UI_EVENT_TOKEN").ok()?;
        let surface = EventSurface::from_wire(&std::env::var("AC_UI_EVENT_SURFACE").ok()?)?;
        let socket = UdpSocket::bind("127.0.0.1:0").ok()?;
        socket.set_nonblocking(true).ok()?;
        Some((socket, address, token, surface))
    });
    if let Some((socket, address, token, surface)) = sender {
        let message = format!("{token}:{}", surface.wire_name());
        let _ = socket.send_to(message.as_bytes(), address);
    }
}
