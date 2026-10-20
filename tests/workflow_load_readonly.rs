//! #65: `workflow::load` is a pure read. It must never change the run database (schema or
//! journal mode) or take a write lock, so polling it cannot make a starting run fail with
//! "database is locked". The timing race itself is covered by `active_feedback`.
use agentconductor::workflow;

#[test]
fn load_never_changes_the_run_database() {
    let dir = std::env::temp_dir().join(format!(
        "ac-load-readonly-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    // A database file that exists but is not initialized yet: a run is just starting.
    let db = dir.join("state.sqlite");
    drop(rusqlite::Connection::open(&db).unwrap());

    assert!(workflow::load(&dir).is_err(), "nothing to load yet");

    let connection = rusqlite::Connection::open(&db).unwrap();
    let objects: i64 = connection
        .query_row("SELECT count(*) FROM sqlite_master", [], |row| row.get(0))
        .unwrap();
    assert_eq!(objects, 0, "load must not create the schema");
    drop(connection);
    std::fs::remove_dir_all(dir).unwrap();
}
