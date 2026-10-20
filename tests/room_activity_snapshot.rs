#![cfg(unix)]
//! GitHub #100: the room snapshot carries per-task live activity (empty when nothing runs).
use agentconductor::{ui::Room, workflow};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn the_snapshot_has_an_activity_map_that_is_empty_when_no_task_runs() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let room = Room::open(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(snapshot["activity"].as_object().unwrap().is_empty());
    assert!(room.document().unwrap().contains("id=\"task-activity\""));
    std::fs::remove_dir_all(root).unwrap();
}
