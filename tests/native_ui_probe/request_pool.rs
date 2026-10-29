use super::RequestWorker;
use eframe::egui;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

#[test]
fn slow_read_does_not_block_the_state_change_lane() {
    let (started, started_rx) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));
    let dispatch = Arc::new(move |packet: &[u8]| {
        if packet == br#"{"kind":"snapshot"}"# {
            started.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
        }
        Ok(serde_json::json!({"request": String::from_utf8_lossy(packet)}))
    });
    let worker = RequestWorker::connect_with(egui::Context::default(), false, dispatch).unwrap();

    worker.send(br#"{"kind":"snapshot"}"#).unwrap();
    started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    worker.send(br#"{"kind":"action"}"#).unwrap();

    assert_eq!(
        worker
            .replies
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap()["request"],
        r#"{"kind":"action"}"#
    );
    release.send(()).unwrap();
    assert!(
        worker
            .replies
            .recv_timeout(Duration::from_millis(50))
            .is_err()
    );
}
