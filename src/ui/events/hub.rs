use super::{UiEvent, UiSurface, bridge::EventBridge, recovery::RecoveryObserver};
use eframe::egui;
#[cfg(test)]
use std::net::SocketAddr;
use std::{
    process::Command,
    sync::{Arc, Mutex, atomic::Ordering, mpsc},
};

#[derive(Clone)]
pub struct EventHub {
    pub(super) subscribers: Arc<Mutex<Vec<Subscriber>>>,
    pub(super) mutations: Arc<Mutex<()>>,
    pub(super) publisher: EventPublisher,
    capacity: usize,
    bridge: Option<Arc<EventBridge>>,
    recovery: Option<Arc<RecoveryObserver>>,
}

#[derive(Clone)]
pub(super) struct EventPublisher {
    pending: Arc<std::sync::atomic::AtomicU8>,
    wake: mpsc::SyncSender<()>,
    subscribers: Arc<Mutex<Vec<Subscriber>>>,
}

#[derive(Clone)]
pub(super) struct Subscriber {
    surface: UiSurface,
    context: egui::Context,
    sender: mpsc::SyncSender<UiEvent>,
}

impl EventPublisher {
    pub(super) fn publish(&self, event: UiEvent) {
        self.pending.fetch_or(event_mask(event), Ordering::Release);
        if matches!(
            self.wake.try_send(()),
            Err(mpsc::TrySendError::Disconnected(()))
        ) {
            deliver_nonblocking(&self.subscribers, event);
        }
    }
}

impl EventHub {
    pub fn new(capacity: usize) -> Self {
        let subscribers = Arc::new(Mutex::new(Vec::new()));
        let pending = Arc::new(std::sync::atomic::AtomicU8::new(0));
        let (wake, events) = mpsc::sync_channel(1);
        let dispatch_subscribers = Arc::clone(&subscribers);
        let dispatch_pending = Arc::clone(&pending);
        let _ = std::thread::Builder::new()
            .name("native-ui-event-fanout".into())
            .spawn(move || {
                while events.recv().is_ok() {
                    deliver_pending(&dispatch_subscribers, &dispatch_pending);
                }
            });
        let publisher = EventPublisher {
            pending,
            wake,
            subscribers: Arc::clone(&subscribers),
        };
        let bridge = EventBridge::new(publisher.clone()).ok().map(Arc::new);
        let recovery = if bridge.is_none() {
            RecoveryObserver::new(publisher.clone()).ok().map(Arc::new)
        } else {
            None
        };
        Self {
            subscribers,
            mutations: Arc::new(Mutex::new(())),
            publisher,
            capacity: capacity.max(1),
            bridge,
            recovery,
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
        self.publisher.publish(event);
    }

    pub fn watch_run_dir(&self, dir: &std::path::Path, surface: UiSurface) {
        if let Some(bridge) = &self.bridge {
            bridge.watch_run_dir(dir, surface);
        } else if let Some(recovery) = &self.recovery {
            recovery.watch_run_dir(dir, surface);
        }
    }

    pub(crate) fn configure_child(&self, command: &mut Command, surface: UiSurface) {
        if let Some(bridge) = &self.bridge {
            bridge.apply_to(command, surface);
        }
    }

    #[cfg(test)]
    pub(super) fn bridge_credentials(&self) -> Option<(SocketAddr, String)> {
        self.bridge.as_ref().map(|bridge| bridge.credentials())
    }
}

fn event_mask(event: UiEvent) -> u8 {
    match event.surface {
        UiSurface::Workspace => 1,
        UiSurface::AgentRoom => 2,
        UiSurface::Both => 3,
    }
}

fn deliver_pending(
    subscribers: &Arc<Mutex<Vec<Subscriber>>>,
    pending: &std::sync::atomic::AtomicU8,
) {
    let pending = pending.swap(0, Ordering::AcqRel);
    if pending == 3 {
        deliver(subscribers, UiEvent::invalidated(UiSurface::Both));
    } else if pending & 1 != 0 {
        deliver(subscribers, UiEvent::invalidated(UiSurface::Workspace));
    } else if pending & 2 != 0 {
        deliver(subscribers, UiEvent::invalidated(UiSurface::AgentRoom));
    }
}

fn deliver(subscribers: &Arc<Mutex<Vec<Subscriber>>>, event: UiEvent) {
    let mut subscribers = subscribers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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

fn deliver_nonblocking(subscribers: &Arc<Mutex<Vec<Subscriber>>>, event: UiEvent) {
    let Ok(mut subscribers) = subscribers.try_lock() else {
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

impl Default for EventHub {
    fn default() -> Self {
        Self::new(1)
    }
}
