use crate::ui::events::{
    Event, EventBus, EventSurface,
    listener::{RunChangeListener, StartupWatch},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[test]
fn report_change_during_workspace_wakeup_is_not_swallowed() {
    let root = std::env::temp_dir().join(format!("event-wakeup-race-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let startup = root.join("startup.json");
    let report = root.join("report.json");
    std::fs::write(&startup, br#"{"job":"idle"}"#).unwrap();
    std::fs::write(&report, br#"{"state":"running","completed":0}"#).unwrap();

    let report_on_wakeup = report.clone();
    let changed = Arc::new(AtomicBool::new(false));
    let change_once = Arc::clone(&changed);
    let bus = Arc::new(EventBus::default());
    let workspace = bus
        .subscribe_with_wakeup(EventSurface::Workspace, move || {
            if !change_once.swap(true, Ordering::SeqCst) {
                std::fs::write(&report_on_wakeup, br#"{"state":"running","completed":10}"#)
                    .unwrap();
            }
        })
        .unwrap();
    let room = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let watch = StartupWatch {
        startup_path: startup.clone(),
        report_path: Some(report),
    };
    let listener = RunChangeListener::spawn_startup_with_interval(
        move || Some(watch.clone()),
        bus,
        std::time::Duration::from_millis(10),
    )
    .unwrap();

    std::fs::write(&startup, br#"{"job":"running"}"#).unwrap();
    assert_eq!(
        workspace
            .events
            .recv_timeout(std::time::Duration::from_secs(2)),
        Ok(Event::WorkspaceChanged)
    );
    assert_eq!(
        workspace
            .events
            .recv_timeout(std::time::Duration::from_secs(2)),
        Ok(Event::RunChanged)
    );
    assert_eq!(
        room.events.recv_timeout(std::time::Duration::from_secs(2)),
        Ok(Event::RunChanged)
    );

    drop(listener);
    std::fs::remove_dir_all(root).unwrap();
}
