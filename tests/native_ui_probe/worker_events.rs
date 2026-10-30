use crate::{
    native_ui_probe::RequestWorker,
    ui::events::{EventHub, UiEvent, UiSurface},
};
use std::sync::Arc;

#[test]
fn subscribed_surface_wakes_when_its_state_changes() {
    let context = eframe::egui::Context::default();
    let hub = EventHub::default();
    let subscriber = hub.subscribe(UiSurface::Workspace, context.clone());
    let mut worker = RequestWorker::connect_with(
        context.clone(),
        false,
        Arc::new(|_| Ok(serde_json::Value::Null)),
    )
    .unwrap();
    worker.events = subscriber;

    hub.publish(UiEvent::invalidated(UiSurface::Workspace));

    assert_eq!(
        worker
            .events
            .recv_timeout(std::time::Duration::from_secs(1)),
        Ok(UiEvent::invalidated(UiSurface::Workspace))
    );
}

#[test]
fn another_surface_does_not_wake_this_listener() {
    let context = eframe::egui::Context::default();
    let hub = EventHub::default();
    let subscriber = hub.subscribe(UiSurface::Workspace, context.clone());
    let mut worker = RequestWorker::connect_with(
        context.clone(),
        false,
        Arc::new(|_| Ok(serde_json::Value::Null)),
    )
    .unwrap();
    worker.events = subscriber;

    hub.publish(UiEvent::invalidated(UiSurface::AgentRoom));

    assert!(matches!(
        worker
            .events
            .recv_timeout(std::time::Duration::from_millis(30)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));
}
