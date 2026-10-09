use agentconductor::state::{Status, Store};

#[test]
fn transitions_are_audited_and_invalid_completion_is_rejected() {
    let mut db = Store::open(":memory:").unwrap();
    db.create("a").unwrap();
    assert!(db.transition("a", Status::Completed).is_err());
    for next in [
        Status::Ready,
        Status::Starting,
        Status::Running,
        Status::Completed,
    ] {
        db.transition("a", next).unwrap();
        assert_eq!(db.status("a").unwrap(), next);
    }
    assert_eq!(db.events("a").unwrap().len(), 4);
    assert!(db.transition("a", Status::Running).is_err());
}

#[test]
fn stop_requires_a_termination_transition_before_resume() {
    let mut db = Store::open(":memory:").unwrap();
    db.create("a").unwrap();
    for next in [
        Status::Ready,
        Status::Starting,
        Status::Running,
        Status::Stopping,
        Status::Stopped,
        Status::Ready,
    ] {
        db.transition("a", next).unwrap();
    }
    assert_eq!(db.status("a").unwrap(), Status::Ready);
}

#[test]
fn replayed_events_are_idempotent_and_unknown_runs_are_rejected() {
    let db = Store::open(":memory:").unwrap();
    db.create("a").unwrap();
    assert!(db.append("a", 10, "progress", "{}").unwrap());
    assert!(!db.append("a", 10, "progress", "{}").unwrap());
    assert!(db.append("missing", 10, "progress", "{}").is_err());
}

#[test]
fn state_survives_reopening_database() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ac-state-{}-{stamp}.sqlite", std::process::id()));
    {
        let mut db = Store::open(&path).unwrap();
        db.create("persist").unwrap();
        db.transition("persist", Status::Ready).unwrap();
    }
    let db = Store::open(&path).unwrap();
    assert_eq!(db.status("persist").unwrap(), Status::Ready);
    drop(db);
    std::fs::remove_file(path).unwrap();
}
