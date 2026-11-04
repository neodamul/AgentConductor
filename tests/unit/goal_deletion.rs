//! Slow filesystem deletion must not hold the startup settings mutex.
use super::*;

#[test]
fn snapshots_remain_available_while_competing_mutations_are_refused() {
    let root = std::env::temp_dir().join(format!(
        "ac-goal-delete-gate-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let startup =
        std::sync::Arc::new(Startup::open(&root, &std::env::current_exe().unwrap()).unwrap());
    let gate = startup.goal_deletion.lock().unwrap();
    let worker = startup.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let thread = std::thread::spawn(move || {
        let snapshot = worker.dispatch(br#"{"kind":"snapshot"}"#);
        let mutation = worker.dispatch(br#"{"kind":"new_goal","revision":"ignored"}"#);
        tx.send((snapshot, mutation)).unwrap();
    });
    let answer = rx.recv_timeout(std::time::Duration::from_secs(2));
    drop(gate);
    thread.join().unwrap();
    let (snapshot, mutation) = answer.expect("snapshot waited for deletion");
    assert!(snapshot.is_ok());
    assert!(mutation.unwrap_err().contains("deletion is in progress"));
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
