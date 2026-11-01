#[test]
fn oversized_settings_leave_no_pending_file_or_partial_update() {
    let root = std::env::temp_dir().join(format!(
        "ac-startup-write-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let path = root.join("startup.json");
    super::write(&path, &"original").unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(
        super::write(&path, &"x".repeat(65536))
            .unwrap_err()
            .contains(crate::ui::messages::text(
                "native.startup_document_exceeds_64_kib"
            ))
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn detached_snapshot_releases_startup_state_lock_before_work() {
    use std::sync::{Arc, Mutex, mpsc};

    let state = Arc::new(Mutex::new(super::Saved {
        imports: None,
        mcp_result: None,
        chat_result: None,
        optimize_result: None,
        optimize_goal: None,
        config: super::defaults().unwrap(),
        proposal: None,
        job: None,
        quota_job: None,
        metadata_queue: Vec::new(),
        metadata_input: None,
        metadata_settled: None,
        metadata_error: None,
        last_run: None,
    }));
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let snapshot_state = state.clone();
    let snapshot_thread = std::thread::spawn(move || {
        let guard = snapshot_state.lock().unwrap();
        super::dispatch::with_snapshot_copy(guard, |detached| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            detached.config.profiles.len()
        })
    });

    started_rx.recv().unwrap();
    assert!(state.try_lock().is_ok());
    release_tx.send(()).unwrap();
    assert_eq!(snapshot_thread.join().unwrap(), 3);
}
