use super::RequestWorker;
use eframe::egui;
use serde_json::json;
use std::{
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

#[test]
fn foreground_dispatch_completes_while_a_periodic_poll_is_blocked() {
    let gate = Arc::new((Mutex::new((false, false)), Condvar::new()));
    let dispatch_gate = gate.clone();
    let dispatch = Arc::new(move |packet: &[u8]| {
        if packet == b"snapshot" {
            let (state, wake) = &*dispatch_gate;
            let mut state = state.lock().unwrap();
            state.0 = true;
            wake.notify_all();
            while !state.1 {
                state = wake.wait(state).unwrap();
            }
        }
        Ok(json!({"request":String::from_utf8_lossy(packet)}))
    });
    let worker = RequestWorker::connect_with(egui::Context::default(), false, dispatch).unwrap();
    assert!(worker.send_poll(b"snapshot").unwrap());
    let (state, wake) = &*gate;
    let state = state.lock().unwrap();
    let (state, timeout) = wake
        .wait_timeout_while(state, Duration::from_secs(2), |state| !state.0)
        .unwrap();
    assert!(!timeout.timed_out(), "snapshot dispatch never started");
    drop(state);

    worker.send(b"command").unwrap();
    let reply = worker
        .replies
        .recv_timeout(Duration::from_secs(1))
        .expect("foreground request must bypass a blocked periodic poll")
        .unwrap();
    assert_eq!(reply["request"], "command");
    assert!(worker.poll_replies.try_recv().is_err());

    let (state, wake) = &*gate;
    state.lock().unwrap().1 = true;
    wake.notify_all();
    assert_eq!(
        worker
            .poll_replies
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .unwrap()["request"],
        "snapshot"
    );
}
