use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

use crate::ui::events::{Event, EventBus, event_for_request};

#[test]
fn state_changes_broadcast_to_other_windows_without_echoing_to_the_sender() {
    let bus = EventBus::default();
    let source = bus.subscribe().unwrap();
    let wakes = Arc::new(AtomicUsize::new(0));
    let listener_wakes = wakes.clone();
    let listener = bus
        .subscribe_with_wakeup(move || {
            listener_wakes.fetch_add(1, Ordering::Relaxed);
        })
        .unwrap();
    bus.publish(Some(source.id), Event::StateChanged);

    assert!(matches!(
        source.events.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    assert_eq!(listener.events.try_recv().unwrap(), Event::StateChanged);
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
}

#[test]
fn repeated_state_changes_coalesce_while_a_window_is_busy() {
    let bus = EventBus::default();
    let wakes = Arc::new(AtomicUsize::new(0));
    let listener_wakes = wakes.clone();
    let listener = bus
        .subscribe_with_wakeup(move || {
            listener_wakes.fetch_add(1, Ordering::Relaxed);
        })
        .unwrap();

    for _ in 0..10_000 {
        bus.publish(None, Event::StateChanged);
    }

    assert_eq!(listener.events.try_iter().count(), 1);
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
}

#[test]
fn read_only_snapshots_do_not_trigger_refresh_broadcasts() {
    assert_eq!(event_for_request(br#"{"kind":"snapshot"}"#), None);
    assert_eq!(
        event_for_request(br#"{"kind":"chat"}"#),
        Some(Event::StateChanged)
    );
    assert_eq!(
        event_for_request(br#"{"kind":"future_state_change"}"#),
        Some(Event::StateChanged)
    );
}
