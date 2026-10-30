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
