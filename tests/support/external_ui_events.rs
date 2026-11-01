use super::listener::next_scan_delay;
use super::*;
use std::{process::Command, sync::Arc, time::Duration};

#[test]
fn authenticated_child_event_reaches_only_its_target_surface() {
    let bus = Arc::new(EventBus::default());
    let workspace = bus.subscribe(EventSurface::Workspace).unwrap();
    let room = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let bridge = ProcessEventBridge::new(Arc::clone(&bus)).unwrap();
    let (address, token) = bridge.credentials();

    ProcessEventBridge::send_test_event(address, "wrong-token", EventSurface::Workspace);
    assert_eq!(
        workspace.events.recv_timeout(Duration::from_millis(40)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    );
    ProcessEventBridge::send_test_event(address, &token, EventSurface::Workspace);

    assert_eq!(
        workspace
            .events
            .recv_timeout(Duration::from_secs(1))
            .unwrap(),
        Event::WorkspaceChanged
    );
    assert_eq!(
        room.events.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    );
}

#[test]
fn child_environment_contains_scoped_local_event_endpoint() {
    let bus = Arc::new(EventBus::default());
    let bridge = ProcessEventBridge::new(bus).unwrap();
    let mut command = Command::new("agentconductor");
    bridge
        .endpoint()
        .configure(&mut command, EventSurface::Workspace);
    let (address, _) = bridge.credentials();
    let values = command
        .get_envs()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                value.map(|value| value.to_string_lossy().into_owned()),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    assert_eq!(values["AC_UI_EVENT_ADDRESS"], Some(address.to_string()));
    assert!(values["AC_UI_EVENT_TOKEN"].is_some());
    assert_eq!(values["AC_UI_EVENT_SURFACE"], Some("workspace".into()));
}

#[test]
fn ordinary_provider_commands_remove_parent_event_credentials() {
    let values = crate::subprocess::command("provider")
        .get_envs()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                value.map(|value| value.to_string_lossy().into_owned()),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    for key in [
        "AC_UI_EVENT_ADDRESS",
        "AC_UI_EVENT_TOKEN",
        "AC_UI_EVENT_SURFACE",
    ] {
        assert_eq!(values[key], None, "provider must not inherit {key}");
    }
}

#[test]
fn idle_file_scan_backoff_resets_on_change_and_stops_at_thirty_seconds() {
    let first = next_scan_delay(Duration::from_secs(1), false);
    assert_eq!(first, Duration::from_secs(2));
    assert_eq!(
        next_scan_delay(Duration::from_secs(30), false),
        Duration::from_secs(30)
    );
    assert_eq!(
        next_scan_delay(Duration::from_secs(30), true),
        Duration::from_secs(1)
    );
}
