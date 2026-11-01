use std::sync::Arc;

use crate::native_ui_probe::{Pending, Probe, RequestWorker};
use eframe::egui;

#[test]
fn background_snapshot_updates_visible_state_without_clearing_pending_action() {
    let worker = RequestWorker::with_dispatch_for_test(
        egui::Context::default(),
        Arc::new(|_| {
            Ok(serde_json::json!({
                "revision":"updated", "config":{"goal":"server value"}
            }))
        }),
    )
    .unwrap();
    let mut probe = Probe {
        worker: Some(worker),
        pending: Some(Pending::ChatStart),
        goal: "unsaved input".into(),
        error: Some("keep the current message".into()),
        ..Probe::default()
    };

    probe.refresh_background();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while probe.background_refresh_pending {
        probe.poll();
        assert!(
            std::time::Instant::now() < deadline,
            "background reply timed out"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert!(matches!(probe.pending, Some(Pending::ChatStart)));
    assert_eq!(probe.snapshot.as_ref().unwrap()["revision"], "updated");
    assert_eq!(probe.goal, "unsaved input");
    assert_eq!(probe.error.as_deref(), Some("keep the current message"));
}
