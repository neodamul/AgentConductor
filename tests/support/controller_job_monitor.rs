use super::*;
use crate::{process::Ownership, state::Store, ui::controller::Job};
use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
};

#[test]
fn controller_poll_stays_responsive_while_terminal_audit_is_locked() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-job-monitor-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let logs = root.join("process");
    fs::create_dir_all(&logs).unwrap();
    let database = root.join("state.sqlite");
    Store::open(&database).unwrap();
    let writer = rusqlite::Connection::open(&database).unwrap();
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    let exe = std::env::current_exe().unwrap();
    let child = Command::new(&exe)
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let owner = Ownership {
        coordinator_pid: std::process::id(),
        pid: Some(child.id()),
        process_group: i32::try_from(child.id()).ok(),
        phase: "running".into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    };
    record_owner(&logs, &owner).unwrap();
    let controller = Controller {
        dir: root.clone(),
        exe,
        job: std::sync::Arc::new(Mutex::new(Some(Job {
            id: "job".into(),
            label: "test".into(),
            child: std::sync::Arc::new(Mutex::new(child)),
            logs: logs.clone(),
            owner,
            exit: None,
            finished: false,
            monitoring: false,
            audit_error: None,
        }))),
    };
    controller.start_job_monitor().unwrap();

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while serde_json::from_slice::<Ownership>(&fs::read(logs.join("owner.json")).unwrap())
        .unwrap()
        .phase
        != "terminal"
    {
        assert!(std::time::Instant::now() < deadline, "child did not exit");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(controller.poll().unwrap()["state"], "running");
    writer.execute_batch("COMMIT").unwrap();

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while controller.poll().unwrap()["state"] == "running" {
        assert!(std::time::Instant::now() < deadline, "audit did not finish");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(controller.poll().unwrap()["state"], "completed");
    drop(controller.job.lock().unwrap().take());
    drop(writer);
    fs::remove_dir_all(root).unwrap();
}
