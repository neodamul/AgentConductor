use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

use crate::ui::events::{Event, EventBus, EventSurface, event_for_request};

#[test]
fn workspace_changes_only_wake_workspace_listeners_without_echoing_to_the_sender() {
    let bus = EventBus::default();
    let source = bus.subscribe(EventSurface::Workspace).unwrap();
    let workspace_wakes = Arc::new(AtomicUsize::new(0));
    let listener_wakes = workspace_wakes.clone();
    let listener = bus
        .subscribe_with_wakeup(EventSurface::Workspace, move || {
            listener_wakes.fetch_add(1, Ordering::Relaxed);
        })
        .unwrap();
    let room_wakes = Arc::new(AtomicUsize::new(0));
    let listener_wakes = room_wakes.clone();
    let room = bus
        .subscribe_with_wakeup(EventSurface::AgentRoom, move || {
            listener_wakes.fetch_add(1, Ordering::Relaxed);
        })
        .unwrap();
    bus.publish(Some(source.id), Event::WorkspaceChanged);

    assert!(matches!(
        source.events.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    assert_eq!(listener.events.try_recv().unwrap(), Event::WorkspaceChanged);
    assert!(matches!(
        room.events.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    assert_eq!(workspace_wakes.load(Ordering::Relaxed), 1);
    assert_eq!(room_wakes.load(Ordering::Relaxed), 0);
}

#[test]
fn run_changes_wake_both_workspace_and_agent_room_listeners() {
    let bus = EventBus::default();
    let workspace = bus.subscribe(EventSurface::Workspace).unwrap();
    let room = bus.subscribe(EventSurface::AgentRoom).unwrap();

    bus.publish(None, Event::RunChanged);

    assert_eq!(workspace.events.try_recv().unwrap(), Event::RunChanged);
    assert_eq!(room.events.try_recv().unwrap(), Event::RunChanged);
}

#[test]
fn repeated_state_changes_coalesce_while_a_window_is_busy() {
    let bus = EventBus::default();
    let wakes = Arc::new(AtomicUsize::new(0));
    let listener_wakes = wakes.clone();
    let listener = bus
        .subscribe_with_wakeup(EventSurface::Workspace, move || {
            listener_wakes.fetch_add(1, Ordering::Relaxed);
        })
        .unwrap();

    for _ in 0..10_000 {
        bus.publish(None, Event::WorkspaceChanged);
    }

    assert_eq!(listener.events.try_iter().count(), 1);
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
}

#[test]
fn disconnected_subscriber_does_not_block_or_remain_registered() {
    let bus = EventBus::default();
    let disconnected = bus.subscribe(EventSurface::Workspace).unwrap();
    drop(disconnected);
    let listener = bus.subscribe(EventSurface::Workspace).unwrap();

    bus.publish(None, Event::WorkspaceChanged);

    assert_eq!(listener.events.try_recv().unwrap(), Event::WorkspaceChanged);
    assert_eq!(bus.subscribers.lock().unwrap().listeners.len(), 1);
}

#[test]
fn read_only_snapshots_do_not_trigger_refresh_broadcasts() {
    assert_eq!(event_for_request(br#"{"kind":"snapshot"}"#), None);
    assert_eq!(event_for_request(br#"{"kind":"baseline_status"}"#), None);
    assert_eq!(
        event_for_request(br#"{"kind":"chat"}"#),
        Some(Event::WorkspaceChanged)
    );
    assert_eq!(
        event_for_request(br#"{"kind":"future_state_change"}"#),
        Some(Event::WorkspaceChanged)
    );
    assert_eq!(
        event_for_request(br#"{"kind":"action","room":true}"#),
        Some(Event::RunChanged)
    );
    assert_eq!(
        event_for_request(br#"{"kind":"start"}"#),
        Some(Event::RunChanged)
    );
}
