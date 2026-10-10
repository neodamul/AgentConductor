#![cfg(unix)]
use agentconductor::{ui::Room, workflow};
use std::{fs, time::Duration};
#[path = "support/workflow.rs"]
mod support;
fn snapshot(room: &Room) -> serde_json::Value {
    room.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn action(
    room: &Room,
    revision: &serde_json::Value,
    action: serde_json::Value,
) -> Result<serde_json::Value, String> {
    room.dispatch(
        serde_json::json!({"kind":"action","revision":revision,"action":action})
            .to_string()
            .as_bytes(),
    )
}
#[tokio::test]
async fn native_review_requires_current_revision_and_exact_candidate_and_keeps_accepted_work_after_close()
 {
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
    let state = snapshot(&room);
    assert_eq!(state["report"]["state"], "waiting_approval");
    let candidate = state["report"]["candidate"]["commit"]
        .as_str()
        .unwrap()
        .to_owned();
    let approve = serde_json::json!({"kind":"approve","sha":candidate});
    assert!(action(&room, &serde_json::json!("0".repeat(64)), approve.clone()).is_err());
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    action(&room, &state["revision"], approve).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let state = snapshot(&room);
        if state["job"]["state"] == "completed" {
            assert_eq!(state["report"]["state"], "approved");
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let evidence = room.dispatch(br#"{"kind":"evidence"}"#).unwrap();
    assert!(evidence["diff"].as_str().unwrap().contains("one.txt"));
    assert_eq!(evidence["report"]["tests"].as_array().unwrap().len(), 3);
    let state = snapshot(&room);
    assert!(
        action(
            &room,
            &state["revision"],
            serde_json::json!({"kind":"merge","sha":"wrong"})
        )
        .is_err()
    );
    let accepted = action(
        &room,
        &state["revision"],
        serde_json::json!({"kind":"merge","sha":candidate}),
    )
    .unwrap();
    let id = accepted["id"].as_str().unwrap();
    let owner: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(format!("coordinator-ui-{id}/process/owner.json"))).unwrap(),
    )
    .unwrap();
    let pid = owner["pid"].as_i64().unwrap() as libc::pid_t;
    drop(room);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while workflow::load(&dir).unwrap().state != "completed" {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    assert_eq!(libc::WEXITSTATUS(status), 0);
    assert_eq!(
        agentconductor::git::Repository::open(&root.join("repo"))
            .unwrap()
            .head()
            .unwrap(),
        candidate
    );
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn broker_revision_is_rechecked_by_the_child_under_the_workflow_lock() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let revision = workflow::revision(&report).unwrap();
    workflow::revise(&dir, "one.txt", "Additional edge case").unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args([
            "ui-action",
            dir.to_str().unwrap(),
            &revision,
            "approve",
            &report.candidate.unwrap().commit,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("changed since review"));
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn ui_evidence_does_not_publish_profile_environment_or_follow_artifact_aliases() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0]
        .environment
        .insert("INTERNAL_CREDENTIAL".into(), "fixture-private-value".into());
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let server = Room::open(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let evidence = server
        .dispatch(br#"{"kind":"evidence"}"#)
        .unwrap()
        .to_string();
    assert!(!evidence.contains("fixture-private-value"));
    let stdout = workflow::load(&dir).unwrap().tests[0].stdout.clone();
    let outside = root.join("private.log");
    fs::write(&outside, "fixture-private-log").unwrap();
    fs::rename(&stdout, stdout.with_extension("retained")).unwrap();
    std::os::unix::fs::symlink(&outside, &stdout).unwrap();
    let evidence = server.dispatch(br#"{"kind":"evidence"}"#).unwrap_err();
    assert!(!evidence.contains("fixture-private-log"));
    drop(server);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_run_can_have_only_one_native_room_and_release_allows_reopening() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let first = Room::open(&dir, exe).unwrap();
    assert!(Room::open(&dir, exe).is_err());
    let child = std::process::Command::new(exe)
        .args(["ui", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!child.status.success());
    assert!(String::from_utf8_lossy(&child.stderr).contains("Agent Room is already open"));
    drop(first);
    let reopened = Room::open(&dir, exe).unwrap();
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_signalled_coordinator_is_failed_not_running_and_keeps_its_real_signal() {
    use std::os::unix::fs::PermissionsExt;
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let exe = root.join("signal-coordinator");
    fs::write(&exe, "#!/bin/sh\nkill -TERM $$\n").unwrap();
    fs::set_permissions(&exe, fs::Permissions::from_mode(0o700)).unwrap();
    let room = Room::open(&dir, &exe).unwrap();
    for _ in 0..2 {
        let state = snapshot(&room);
        let accepted = action(
            &room,
            &state["revision"],
            serde_json::json!({"kind":"approve","sha":state["report"]["candidate"]["commit"]}),
        )
        .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let job = snapshot(&room)["job"].clone();
            if job["state"] == "failed" {
                assert!(job["exit_code"].is_null());
                assert_eq!(job["termination_signal"], libc::SIGTERM);
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        let owner: agentconductor::process::Ownership = serde_json::from_slice(
            &fs::read(dir.join(format!(
                "coordinator-ui-{}/process/owner.json",
                accepted["id"].as_str().unwrap()
            )))
            .unwrap(),
        )
        .unwrap();
        owner.ensure_quiescent().unwrap();
    }
    drop(room);
    fs::remove_dir_all(root).unwrap();
}
