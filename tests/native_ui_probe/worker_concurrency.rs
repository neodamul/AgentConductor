use std::{
    sync::{Arc, mpsc},
    time::Duration,
};

use super::RequestWorker;
use eframe::egui;
use serde_json::Value;

#[test]
fn background_refresh_does_not_block_interactive_dispatch() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(std::sync::Mutex::new(release_rx));
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(move |packet: &[u8]| {
            let request: Value = serde_json::from_slice(packet).unwrap();
            match request["kind"].as_str() {
                Some("slow_snapshot") => {
                    started_tx.send(()).unwrap();
                    release_rx.lock().unwrap().recv().unwrap();
                    Ok(serde_json::json!({"kind":"snapshot"}))
                }
                Some("save") => Ok(serde_json::json!({"kind":"save","accepted":true})),
                other => panic!("unexpected request: {other:?}"),
            }
        }),
    )
    .unwrap();

    worker
        .send_background(br#"{"kind":"slow_snapshot"}"#)
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.send(br#"{"kind":"save"}"#).unwrap();
    let reply = worker
        .replies
        .recv_timeout(Duration::from_secs(2))
        .expect("interactive request must complete while refresh is blocked")
        .unwrap();
    assert_eq!(reply["kind"], "save");

    release_tx.send(()).unwrap();
    let background = worker
        .background_replies
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap();
    assert_eq!(background["kind"], "snapshot");
}

#[test]
fn slow_interactive_read_does_not_block_independent_interactive_action() {
    let (started_tx, started_rx) = mpsc::channel();
    let (action_tx, action_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(std::sync::Mutex::new(release_rx));
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(move |packet: &[u8]| {
            let request: Value = serde_json::from_slice(packet).unwrap();
            match request["kind"].as_str() {
                Some("snapshot") => {
                    started_tx.send(()).unwrap();
                    release_rx.lock().unwrap().recv().unwrap();
                    Ok(serde_json::json!({"kind":"snapshot"}))
                }
                Some("action") => {
                    action_tx.send(()).unwrap();
                    Ok(serde_json::json!({"kind":"action"}))
                }
                other => panic!("unexpected request: {other:?}"),
            }
        }),
    )
    .unwrap();

    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.send(br#"{"kind":"action"}"#).unwrap();
    action_rx
        .recv_timeout(Duration::from_millis(250))
        .expect("the independent action must reach its worker lane");
    let reply = worker
        .replies
        .recv_timeout(Duration::from_millis(250))
        .expect("a slow read must not block an independent UI action")
        .unwrap();
    assert_eq!(reply["kind"], "action");

    release_tx.send(()).unwrap();
    assert!(
        worker
            .replies
            .recv_timeout(Duration::from_millis(50))
            .is_err(),
        "a stale snapshot reply must not replace the newer action"
    );
}

#[test]
fn bounded_read_lane_runs_two_independent_reads_concurrently() {
    let (started_tx, started_rx) = mpsc::channel();
    let gate = Arc::new(std::sync::Barrier::new(3));
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new({
            let gate = Arc::clone(&gate);
            move |packet: &[u8]| {
                started_tx.send(()).unwrap();
                gate.wait();
                Ok(serde_json::from_slice(packet).unwrap())
            }
        }),
    )
    .unwrap();

    worker.send(br#"{"kind":"snapshot","id":1}"#).unwrap();
    worker.send(br#"{"kind":"snapshot","id":2}"#).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    started_rx
        .recv_timeout(Duration::from_millis(250))
        .expect("the second read must start on the bounded read lane while the first is waiting");
    gate.wait();
    assert_eq!(
        worker
            .replies
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap()["id"],
        2
    );
}

#[test]
fn state_changes_remain_serialized_while_reads_use_parallel_workers() {
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(std::sync::Mutex::new(release_rx));
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(move |packet: &[u8]| {
            let request: Value = serde_json::from_slice(packet).unwrap();
            started_tx.send(request["id"].as_u64().unwrap()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
            Ok(request)
        }),
    )
    .unwrap();

    worker.send(br#"{"kind":"action","id":1}"#).unwrap();
    assert_eq!(started_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 1);
    worker.send(br#"{"kind":"action","id":2}"#).unwrap();
    assert!(started_rx.recv_timeout(Duration::from_millis(50)).is_err());
    release_tx.send(()).unwrap();
    assert_eq!(started_rx.recv_timeout(Duration::from_secs(2)).unwrap(), 2);
    release_tx.send(()).unwrap();
}

#[test]
fn slow_external_io_does_not_block_workspace_actions() {
    let (started_tx, started_rx) = mpsc::channel();
    let (action_tx, action_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(std::sync::Mutex::new(release_rx));
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(move |packet: &[u8]| {
            let request: Value = serde_json::from_slice(packet).unwrap();
            match request["kind"].as_str() {
                Some("test_messenger") => {
                    started_tx.send(()).unwrap();
                    release_rx.lock().unwrap().recv().unwrap();
                    Ok(serde_json::json!({"sent": true}))
                }
                Some("save") => {
                    action_tx.send(()).unwrap();
                    Ok(serde_json::json!({"saved": true}))
                }
                other => panic!("unexpected request: {other:?}"),
            }
        }),
    )
    .unwrap();

    worker.send(br#"{"kind":"test_messenger"}"#).unwrap();
    started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.send(br#"{"kind":"save"}"#).unwrap();
    action_rx
        .recv_timeout(Duration::from_millis(250))
        .expect("slow external I/O must not occupy the workspace mutation lane");
    let reply = worker
        .replies
        .recv_timeout(Duration::from_millis(250))
        .expect("the workspace action should return while external I/O is pending")
        .unwrap();
    assert_eq!(reply["saved"], true);
    release_tx.send(()).unwrap();
}
