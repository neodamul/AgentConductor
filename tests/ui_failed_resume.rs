#![cfg(unix)]
use agentconductor::{ui::Room, workflow};
use std::{fs, path::Path};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_scope_failure_resume_points_to_recovery_before_a_ui_job_is_started() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let mut store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    let mut report = store.document("workflow").unwrap();
    report["state"] = "failed".into();
    // GitHub #625: only a scope failure is still refused (it has its own recovery); any other failed run is resumed.
    report["error"] = "out-of-scope change: other.txt".into();
    store
        .record("workflow", &report, "failed run fixture")
        .unwrap();

    let room = Room::open(&dir, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let response = room.dispatch(
        serde_json::json!({
            "kind":"action",
            "revision":snapshot["revision"],
            "action":{"kind":"resume"}
        })
        .to_string()
        .as_bytes(),
    );

    let error = response.expect_err("a scope failure must not launch resume-due");
    assert_eq!(
        error,
        agentconductor::ui::messages::text("native.scope_failure_use_recovery")
    );
    assert!(fs::read_dir(&dir).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("coordinator-ui-")
    }));
    drop(room);
    fs::remove_dir_all(root).unwrap();
}
