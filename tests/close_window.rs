#![cfg(unix)]
//! R-118 / DD-127: closing the window while a run works asks first. A run keeps going unless
//! the operator chooses to stop it; only a run whose coordinator holds the run lock counts.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

#[tokio::test]
async fn closing_asks_only_for_live_runs_and_stops_them_only_when_chosen() {
    let (root, plan) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    let desktop = root.join("home/.agentconductor/desktop");
    std::fs::create_dir_all(&desktop).unwrap();
    let run = desktop.join("run");
    agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let mut store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
    let mut report = store.document("workflow").unwrap();
    report["state"] = "running".into();
    store
        .record("workflow", &report, "test: a working run")
        .unwrap();
    drop(store);
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::startup(&desktop, exe).unwrap();
    // A run marked running whose coordinator is gone does not hold the window open.
    assert!(room.active_runs().is_empty());

    let coordinator =
        agentconductor::leases::Lease::acquire(&run.join("coordinator.lock")).unwrap();
    let live = room.active_runs();
    assert_eq!(live.len(), 1, "{live:?}");
    assert!(!room.closing());
    let kept = room
        .dispatch(br#"{"kind":"close_window","stop":false}"#)
        .unwrap();
    assert!(room.closing() && kept["stopped"].as_array().unwrap().is_empty());
    assert!(
        !run.join("stop.request").exists(),
        "keep running is the default"
    );
    drop(room);

    let room = Room::startup(&desktop, exe).unwrap();
    let stopped = room
        .dispatch(br#"{"kind":"close_window","stop":true}"#)
        .unwrap();
    assert!(room.closing());
    assert_eq!(stopped["stopped"].as_array().unwrap().len(), 1);
    assert!(
        run.join("stop.request").exists(),
        "the coordinator receives a durable stop request"
    );
    drop(room);
    drop(coordinator);
    drop(env);
    let _ = std::fs::remove_dir_all(root);
}
