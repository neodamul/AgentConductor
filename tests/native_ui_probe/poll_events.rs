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
        pending: Some(Pending::Action),
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
