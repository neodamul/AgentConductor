#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::RequestWorker;
use agentconductor::{ui::Room, workflow};
use serde_json::{Value, json};
use std::{path::Path, sync::Arc, time::Duration};

#[path = "support/workflow.rs"]
mod support;

#[test]
fn room_requests_are_marked_for_the_run_dispatcher() {
    let packet = RequestWorker::route_room_request(br#"{"kind":"evidence"}"#).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&packet).unwrap(),
        json!({"kind":"evidence","room":true})
    );
}

#[test]
fn room_routing_refuses_non_object_packets() {
    assert!(RequestWorker::route_room_request(b"[]").is_err());
}

#[tokio::test]
async fn workspace_worker_switches_to_room_routing_after_open_run() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let room = Arc::new(Room::open(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap());
    let mut worker = RequestWorker::new(room, eframe::egui::Context::default()).unwrap();
    worker.set_room_view(true);
    worker.send(br#"{"kind":"task","index":0}"#).unwrap();
    let reply = worker
        .replies
        .recv_timeout(Duration::from_secs(2))
        .unwrap()
        .unwrap();
    assert_eq!(reply["task"]["id"], "one.txt");
    drop(worker);
    std::fs::remove_dir_all(root).unwrap();
}
