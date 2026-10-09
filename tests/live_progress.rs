#![cfg(unix)]
use agentconductor::{
    process::{self, ProcessSpec},
    protocol::Provider,
    state::Store,
};
use std::{fs, time::Duration};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn normalized_progress_is_queryable_before_process_completion_and_drains_final_records() {
    let (root, _) = support::setup("shared");
    let logs = root.join("logs");
    let db = root.join("state.sqlite");
    let spec = ProcessSpec {
        executable: support::fixture(),
        args: vec!["live-events".into()],
        cwd: root.clone(),
        input: vec![],
        log_dir: logs.clone(),
        timeout: Duration::from_secs(5),
        signal_grace: Duration::from_millis(20),
        environment: Default::default(),
    };
    let read_db = db.clone();
    let release = root.join("release");
    let spool = logs.join("stdout.jsonl");
    let observer = agentconductor::progress::observe(
        process::run_until(spec, None),
        &spool,
        &db,
        "task",
        1,
        Provider::Custom,
    );
    let reader = async move {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
        loop {
            if read_db.exists()
                && let Ok(store) = Store::open(&read_db)
                && store.task_events("task", 1).unwrap().len() >= 2
            {
                assert!(!release.exists());
                fs::write(&release, "release").unwrap();
                break;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "no live progress before completion"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    let (result, ()) = tokio::join!(observer, reader);
    assert_eq!(result.unwrap().unwrap().exit_code, Some(0));
    let store = Store::open(&db).unwrap();
    let events = store.task_events("task", 1).unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[2]["kind"], "result");
    assert_eq!(events[2]["session_id"], "live-session");
    assert!(!store.recall("intermediate").unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn malformed_progress_cancels_the_owned_process_instead_of_waiting_for_timeout() {
    let (root, _) = support::setup("shared");
    let logs = root.join("logs");
    let spec = ProcessSpec {
        executable: support::fixture(),
        args: vec!["live-invalid".into()],
        cwd: root.clone(),
        input: vec![],
        log_dir: logs.clone(),
        timeout: Duration::from_secs(60),
        signal_grace: Duration::from_millis(20),
        environment: Default::default(),
    };
    let result = tokio::time::timeout(
        Duration::from_secs(3),
        agentconductor::progress::observe(
            process::run_until(spec, None),
            &logs.join("stdout.jsonl"),
            &root.join("state.sqlite"),
            "task",
            1,
            Provider::Custom,
        ),
    )
    .await;
    assert!(result.unwrap().is_err());
    let owner: process::Ownership =
        serde_json::from_slice(&fs::read(logs.join("owner.json")).unwrap()).unwrap();
    for _ in 0..50 {
        if owner.ensure_quiescent().is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    owner.ensure_quiescent().unwrap();
    fs::remove_dir_all(root).unwrap();
}
