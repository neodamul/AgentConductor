use crate::{
    process::{Ownership, record_owner},
    state::Store,
    ui::{
        controller::{Controller, Job},
        events::{EventHub, UiEvent, UiSurface},
    },
};
use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
};

#[test]
fn controller_publishes_once_when_a_job_reaches_terminal_state() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-ui-event-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let logs = root.join("process");
    fs::create_dir_all(&logs).unwrap();
    Store::open(root.join("state.sqlite")).unwrap();
    let events = EventHub::default();
    let subscriber = events.subscribe(UiSurface::AgentRoom, eframe::egui::Context::default());
    let controller = Controller::new(
        root.clone(),
        std::env::current_exe().unwrap(),
        events,
        UiSurface::AgentRoom,
    );
    let child = Command::new(controller.exe.clone())
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
    *controller.job.lock().unwrap() = Some(Job {
        id: "job".into(),
        label: "test".into(),
        child: std::sync::Arc::new(std::sync::Mutex::new(child)),
        logs,
        owner,
        exit: None,
        finished: false,
    });
    controller.monitor_job().unwrap();

    assert_eq!(
        subscriber.recv_timeout(Duration::from_secs(1)),
        Ok(UiEvent::invalidated(UiSurface::AgentRoom))
    );
    assert_eq!(controller.poll().unwrap()["state"], "completed");
    assert!(subscriber.try_recv().is_err());
    drop(controller.job.lock().unwrap().take());
    fs::remove_dir_all(root).unwrap();
}
