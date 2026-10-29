use crate::ui::events::{EventHub, UiEvent, UiSurface};
use eframe::egui;

#[test]
fn committed_state_event_reaches_subscribers_without_waiting_for_readers() {
    let events = EventHub::new(1);
    let subscriber = events.subscribe(UiSurface::Workspace, egui::Context::default());
    let event = UiEvent::invalidated(UiSurface::Workspace);

    events.publish(event);

    assert_eq!(subscriber.try_recv(), Ok(event));
}

#[test]
fn slow_and_disconnected_subscribers_do_not_block_event_producers() {
    let events = EventHub::new(1);
    let slow = events.subscribe(UiSurface::Workspace, egui::Context::default());
    let first = UiEvent::invalidated(UiSurface::Workspace);
    events.publish(first);
    events.publish(first);

    assert_eq!(slow.try_recv(), Ok(first));
    assert!(slow.try_recv().is_err());
    drop(slow);
    events.publish(first);
}

#[test]
fn read_only_snapshots_do_not_publish_invalidation_events() {
    assert!(UiEvent::for_request(br#"{"kind":"snapshot","room":true}"#).is_none());
}

#[test]
fn mutations_invalidate_only_the_requesting_surface() {
    assert_eq!(
        UiEvent::for_request(br#"{"kind":"action","room":true}"#),
        Some(UiEvent::invalidated(UiSurface::AgentRoom))
    );
}

#[test]
fn authenticated_child_event_wakes_only_target_surface_and_rejects_forgery() {
    let hub = EventHub::default();
    let workspace = hub.subscribe(UiSurface::Workspace, egui::Context::default());
    let room = hub.subscribe(UiSurface::AgentRoom, egui::Context::default());
    let (address, token) = hub.bridge_credentials().unwrap();

    super::notify_external(address, "invalid-token", UiSurface::Both);
    assert!(
        workspace
            .recv_timeout(std::time::Duration::from_millis(30))
            .is_err()
    );
    super::notify_external(address, &token, UiSurface::AgentRoom);

    assert_eq!(
        room.recv_timeout(std::time::Duration::from_secs(1)),
        Ok(UiEvent::invalidated(UiSurface::AgentRoom))
    );
    assert!(workspace.try_recv().is_err());
}
