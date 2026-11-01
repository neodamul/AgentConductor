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

#[test]
fn run_listener_observes_changes_in_declared_worktree_scope() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-ui-scope-event-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let run = root.join("run");
    let source = root.join("worktree/src");
    std::fs::create_dir_all(&run).unwrap();
    std::fs::create_dir_all(&source).unwrap();
    let file = source.join("lib.rs");
    std::fs::write(&file, "one").unwrap();
    std::fs::write(
        run.join("report.json"),
        serde_json::json!({
            "tasks":[{"id":"task-1","cwd":root.join("worktree")}],
            "plan":{"tasks":[{"id":"task-1","paths":["src"]}]}
        })
        .to_string(),
    )
    .unwrap();
    let bus = Arc::new(EventBus::default());
    let subscription = bus.subscribe(EventSurface::AgentRoom).unwrap();
    let _listener = RunChangeListener::spawn(run.join("report.json"), bus).unwrap();
    std::fs::write(file, "updated contents").unwrap();

    assert_eq!(
        subscription
            .events
            .recv_timeout(std::time::Duration::from_secs(3)),
        Ok(Event::RunChanged)
    );
    std::fs::remove_dir_all(root).unwrap();
}
