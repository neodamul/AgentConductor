#![cfg(unix)]
//! GitHub #122: the Agent Room reads the scope failure, previews a change and applies it.
#[path = "support/workflow.rs"]
mod support;

use agentconductor::{ui::Room, workflow};
use serde_json::json;

#[tokio::test]
async fn the_room_reviews_and_applies_a_scope_recovery() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks.truncate(1);
    plan.tasks[0].prompt = "extra.txt".into();
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let room = Room::open(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();

    let failure = room.dispatch(br#"{"kind":"scope_failure"}"#).unwrap();
    assert_eq!(failure["offending"], json!(["extra.txt"]));
    let preview = room
        .dispatch(json!({"kind":"scope_preview","task":"one.txt","paths":["extra.txt"],"test_command":["true"]}).to_string().as_bytes())
        .unwrap();
    assert_eq!(preview["paths"], json!(["extra.txt"]));

    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let action = json!({"kind":"action","revision":snapshot["revision"],"action":{
        "kind":"scope_recovery","task":"one.txt","paths":["extra.txt"],"test_command":["true"],"digest":preview["digest"]}});
    room.dispatch(action.to_string().as_bytes()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let state = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if state["job"]["state"] == "completed" {
            assert_eq!(state["report"]["state"], "stopped");
            assert_eq!(
                state["report"]["plan"]["tasks"][0]["paths"],
                json!(["extra.txt"])
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{state}");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn resuming_a_scope_failure_points_to_the_recovery_review() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks.truncate(1);
    plan.tasks[0].prompt = "extra.txt".into();
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let room = Room::open(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let resume =
        json!({"kind":"action","revision":snapshot["revision"],"action":{"kind":"resume"}});
    let error = room.dispatch(resume.to_string().as_bytes()).unwrap_err();
    // The message follows the saved interface language.
    assert!(
        error.contains("Review recovery plan") || error.contains("복구 계획 검토"),
        "{error}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
