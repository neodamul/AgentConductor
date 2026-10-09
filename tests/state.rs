use agentconductor::state::{Status, Store};

#[test]
fn normalized_attempts_are_idempotent_and_do_not_transmit_raw_messages_to_memory() {
    use agentconductor::protocol::{Decoder, Provider};
    let mut db = Store::open(":memory:").unwrap();
    let events = Decoder::new(Provider::Custom, 1024)
        .push(b"{\"type\":\"progress\",\"text\":\"private-provider-content\"}\n")
        .unwrap();
    db.append_task_events("task", 1, &events).unwrap();
    let outbox = db.pending_memory().unwrap();
    db.append_task_events("task", 1, &events).unwrap();
    assert_eq!(db.pending_memory().unwrap(), outbox);
    assert_eq!(db.task_events("task", 1).unwrap().len(), 1);
    assert!(!outbox[0].1.contains("private-provider-content"));
    assert!(db.recall("private-provider-content").unwrap().is_empty());
    db.append_task_events("task", 2, &events).unwrap();
    assert_eq!(db.task_events("task", 2).unwrap().len(), 1);
    let mut changed = events;
    changed[0].payload["text"] = serde_json::json!("altered");
    assert!(db.append_task_events("task", 1, &changed).is_err());
}

#[test]
fn progress_queries_bound_large_payloads_without_changing_persisted_evidence() {
    use agentconductor::protocol::{Decoder, Provider};
    let mut db = Store::open(":memory:").unwrap();
    let mut line =
        serde_json::to_vec(&serde_json::json!({"type":"progress","text":"x".repeat(400_000)}))
            .unwrap();
    let mut decoder = Decoder::new(Provider::Custom, 1024 * 1024);
    line.push(b'\n');
    for _ in 0..4 {
        let events = decoder.push(&line).unwrap();
        db.append_task_events("task", 1, &events).unwrap();
    }
    let values = db.task_events("task", 1).unwrap();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["seq"], 2);
    assert_eq!(values[1]["seq"], 3);
}

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
