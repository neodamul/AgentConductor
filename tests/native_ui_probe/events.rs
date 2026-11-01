use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};

use crate::ui::events::{
    Event, EventBus, EventSurface, event_for_request,
    listener::{RunChangeListener, StartupWatch},
};

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
fn subscriber_wakeup_runs_outside_the_registry_lock() {
    let bus = std::sync::Arc::new(EventBus::default());
    let callback_bus = bus.clone();
    let listener = bus
        .subscribe_with_wakeup(EventSurface::Workspace, move || {
            assert!(callback_bus.subscribers.try_lock().is_ok());
        })
        .unwrap();

    bus.publish(None, Event::WorkspaceChanged);

    assert_eq!(listener.events.try_recv().unwrap(), Event::WorkspaceChanged);
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

#[test]
fn persisted_report_changes_emit_a_run_event() {
    let path = std::env::temp_dir().join(format!(
        "agentconductor-event-listener-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let bus = Arc::new(EventBus::default());
    let subscription = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let listener = RunChangeListener::spawn(path.clone(), bus).unwrap();

    std::fs::write(&path, br#"{"state":"running","completed":0}"#).unwrap();

    assert_eq!(
        subscription
            .events
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        Event::RunChanged
    );
    assert!(matches!(
        subscription
            .events
            .recv_timeout(std::time::Duration::from_millis(100)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));

    drop(listener);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn startup_changes_wake_workspace_and_run_changes_wake_both_surfaces() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-startup-listener-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let startup = root.join("startup.json");
    let report = root.join("report.json");
    std::fs::write(&startup, br#"{"job":"idle"}"#).unwrap();
    std::fs::write(&report, br#"{"state":"running","completed":0}"#).unwrap();

    let bus = Arc::new(EventBus::default());
    let workspace = bus.subscribe(EventSurface::Workspace).unwrap();
    let room = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let watched = Arc::new(std::sync::Mutex::new(StartupWatch {
        startup_path: startup.clone(),
        report_path: Some(report.clone()),
    }));
    let current_watch = Arc::clone(&watched);
    let listener =
        RunChangeListener::spawn_startup(move || Some(current_watch.lock().unwrap().clone()), bus)
            .unwrap();

    std::fs::write(&startup, br#"{"job":"running","revision":1}"#).unwrap();
    assert_eq!(
        workspace
            .events
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        Event::WorkspaceChanged
    );
    assert!(matches!(
        room.events.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));

    std::fs::write(&report, br#"{"state":"running","completed":10}"#).unwrap();
    assert_eq!(
        workspace
            .events
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        Event::RunChanged
    );
    assert_eq!(
        room.events
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        Event::RunChanged
    );

    drop(listener);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn startup_listener_follows_workspace_switch_without_owning_old_startup() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-dynamic-listener-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let first_startup = first.join("startup.json");
    let first_report = first.join("report.json");
    let second_startup = second.join("startup.json");
    let second_report = second.join("report.json");
    std::fs::write(&first_startup, br#"{"goal":"first"}"#).unwrap();
    std::fs::write(&first_report, br#"{"state":"running","revision":0}"#).unwrap();
    std::fs::write(&second_startup, br#"{"goal":"second"}"#).unwrap();
    std::fs::write(&second_report, br#"{"state":"running","revision":0}"#).unwrap();

    let bus = Arc::new(EventBus::default());
    let workspace = bus.subscribe(EventSurface::Workspace).unwrap();
    let room = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let current = Arc::new(std::sync::Mutex::new(StartupWatch {
        startup_path: first_startup,
        report_path: Some(first_report.clone()),
    }));
    let watched = Arc::clone(&current);
    let listener =
        RunChangeListener::spawn_startup(move || Some(watched.lock().unwrap().clone()), bus)
            .unwrap();

    *current.lock().unwrap() = StartupWatch {
        startup_path: second_startup,
        report_path: Some(second_report.clone()),
    };
    assert_eq!(
        workspace
            .events
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        Event::WorkspaceChanged
    );
    std::fs::write(&first_report, br#"{"state":"running","revision":1}"#).unwrap();
    assert!(matches!(
        room.events
            .recv_timeout(std::time::Duration::from_millis(700)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));
    std::fs::write(&second_report, br#"{"state":"running","revision":1}"#).unwrap();
    assert_eq!(
        room.events
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap(),
        Event::RunChanged
    );

    drop(listener);
    std::fs::remove_dir_all(root).unwrap();
}
