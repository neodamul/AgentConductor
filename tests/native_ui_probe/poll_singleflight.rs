use super::{Pending, Probe, RequestWorker};
use eframe::egui;
use serde_json::json;
use std::sync::Arc;

#[test]
fn repeated_poll_ticks_coalesce_while_one_poll_is_pending() {
    let dispatch = Arc::new(|_: &[u8]| Ok(json!({"revision":"test"})));
    let worker = RequestWorker::connect_with(egui::Context::default(), false, dispatch).unwrap();
    let mut probe = Probe {
        worker: Some(worker),
        ..Probe::default()
    };

    assert!(
        probe
            .send_poll(b"snapshot", Pending::BackgroundRefresh)
            .unwrap()
    );
    assert!(
        !probe
            .send_poll(b"snapshot", Pending::BackgroundRefresh)
            .unwrap()
    );
}
