use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use eframe::egui;
use serde_json::Value;

use crate::native_ui_probe::{Pending, Probe, RequestWorker};

#[test]
fn state_change_event_refresh_runs_while_an_interactive_request_is_pending() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(move |packet: &[u8]| {
            let request: Value = serde_json::from_slice(packet).unwrap();
            assert_eq!(request["kind"], "snapshot");
            started_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
            Ok(serde_json::json!({"kind": "snapshot"}))
        }),
    )
    .unwrap();
    let mut probe = Probe {
        worker: Some(worker),
        pending: Some(Pending::Action("Running action…")),
        refresh_requested: true,
        ..Probe::default()
    };

    probe.dispatch_background_refresh();
    assert!(
        started_rx.recv_timeout(Duration::from_millis(250)).is_ok(),
        "an unrelated state-change event must start a bounded background refresh while a user action is pending"
    );
    assert!(probe.background_refresh_pending);

    release_tx.send(()).unwrap();
    probe
        .worker
        .as_ref()
        .unwrap()
        .background_replies
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap();
}

#[test]
fn background_snapshot_opens_a_started_run_without_waiting_for_timer() {
    let (opened_tx, opened_rx) = mpsc::channel();
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(move |packet: &[u8]| {
            let request: Value = serde_json::from_slice(packet).unwrap();
            match request["kind"].as_str() {
                Some("snapshot") => Ok(serde_json::json!({
                    "run_available": true,
                    "proposal": {"run": "/run"},
                    "job": {"kind": "start", "state": "running"}
                })),
                Some("open_run") => {
                    opened_tx.send(()).unwrap();
                    Ok(serde_json::json!({"opened": true}))
                }
                other => panic!("unexpected request: {other:?}"),
            }
        }),
    )
    .unwrap();
    let mut probe = Probe {
        worker: Some(worker),
        starting_run: Some("/run".into()),
        refresh_requested: true,
        ..Probe::default()
    };

    probe.dispatch_background_refresh();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while probe.background_refresh_pending && std::time::Instant::now() < deadline {
        probe.poll_background();
        std::thread::sleep(Duration::from_millis(5));
    }

    opened_rx
        .recv_timeout(Duration::from_millis(250))
        .expect("the event-driven snapshot must open the run immediately");
}
