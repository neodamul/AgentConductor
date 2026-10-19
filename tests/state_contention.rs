use agentconductor::state::Store;
use rusqlite::Connection;
use std::{thread, time::Duration};

struct Database(std::path::PathBuf);

impl Drop for Database {
    fn drop(&mut self) {
        for path in [
            self.0.clone(),
            self.0.with_extension("sqlite-wal"),
            self.0.with_extension("sqlite-shm"),
        ] {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[test]
fn task_state_write_waits_for_a_transient_sqlite_writer_lock() {
    let path = Database(std::env::temp_dir().join(format!(
        "ac-state-contention-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    drop(Store::open(&path.0).unwrap());
    let mut store = Store::open(&path.0).unwrap();
    let blocker = Connection::open(&path.0).unwrap();
    blocker.execute_batch("BEGIN IMMEDIATE").unwrap();

    let (started, wait_started) = std::sync::mpsc::channel();
    let writer = thread::spawn(move || {
        started.send(()).unwrap();
        store.record("contention", &serde_json::json!({"ok": true}), "test")
    });
    wait_started.recv().unwrap();
    thread::sleep(Duration::from_millis(2_500));
    blocker.execute_batch("COMMIT").unwrap();

    let result = writer.join().unwrap();
    assert!(
        result.is_ok(),
        "transient writer contention failed: {result:?}"
    );
    assert_eq!(
        Store::open(&path.0)
            .unwrap()
            .document("contention")
            .unwrap(),
        serde_json::json!({"ok": true})
    );
    drop(blocker);
}
