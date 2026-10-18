#![cfg(unix)]
use agentconductor::{ui::Room, workflow};
use std::{fs, path::Path};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn failed_run_resume_is_rejected_before_a_ui_job_is_started() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let mut store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    let mut report = store.document("workflow").unwrap();
    report["state"] = "failed".into();
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

    let error = response.expect_err("failed run must not launch resume-due");
    assert_eq!(
        error,
        agentconductor::ui::messages::text(
            "native.failed_run_cannot_be_resumed_revise_or_reassign"
        )
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
