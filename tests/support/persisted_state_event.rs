use super::{Event, RunChangeListener};
use crate::ui::events::{EventBus, EventSurface};
use std::sync::Arc;

#[test]
fn run_listener_observes_sqlite_changes_when_the_report_is_unchanged() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-ui-state-event-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let report = root.join("report.json");
    std::fs::write(&report, "unchanged").unwrap();
    let bus = Arc::new(EventBus::default());
    let subscription = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let _listener = RunChangeListener::spawn(report, bus).unwrap();
    std::fs::write(root.join("state.sqlite"), "quota updated").unwrap();

    assert_eq!(
        subscription
            .events
            .recv_timeout(std::time::Duration::from_secs(3)),
        Ok(Event::RunChanged)
    );
    std::fs::remove_dir_all(root).unwrap();
}
