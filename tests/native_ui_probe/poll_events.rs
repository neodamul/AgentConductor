use crate::native_ui_probe::{Probe, RequestWorker};
use crate::ui::events::{EventHub, UiEvent, UiSurface};
use eframe::egui;
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[test]
fn metadata_event_refreshes_quota_snapshot_asynchronously() {
    let context = egui::Context::default();
    let hub = EventHub::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let dispatch_calls = Arc::clone(&calls);
    let dispatch = Arc::new(move |packet: &[u8]| {
        let request: serde_json::Value = serde_json::from_slice(packet).unwrap();
        assert_eq!(request["kind"], "snapshot");
        dispatch_calls.fetch_add(1, Ordering::Relaxed);
        Ok(json!({
            "metadata_job":{"state":"completed"},
            "metadata_queue":[],
            "quota_badges":{"codex":{"text":"73%","title":"5h: 73% remaining"}}
        }))
    });
    let mut worker = RequestWorker::connect_with(context.clone(), false, dispatch).unwrap();
    worker.events = hub.subscribe(UiSurface::Workspace, context);
    let mut probe = Probe {
        worker: Some(worker),
        snapshot: Some(json!({
            "metadata_job":{"state":"running"},
            "metadata_queue":[]
        })),
        ..Probe::default()
    };

    hub.publish(UiEvent::invalidated(UiSurface::Workspace));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while probe.snapshot.as_ref().unwrap()["quota_badges"]["codex"]["text"] != "73%" {
        probe.poll();
        assert!(
            std::time::Instant::now() < deadline,
            "metadata event was not applied"
        );
        std::thread::yield_now();
    }

    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(probe.pending.is_none());
    assert!(probe.poll_pending.is_none());
}
