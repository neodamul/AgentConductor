#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_preserves_evidence_blocks_auto_resume_and_explicit_resume_reverifies() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let workdir = dir.clone();
    let repo = root.join("repo");
    let job = tokio::spawn(async move { workflow::run(plan, &repo, &workdir, 10).await });
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let ready = dir.join("task-0/process/stdout.jsonl");
            if std::fs::metadata(ready).is_ok_and(|m| m.len() > 0) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    workflow::stop(&dir, "review partial work").unwrap();
    let report = job.await.unwrap().unwrap();
    assert_eq!(report.state, "stopped");
    assert!(report.tasks.iter().any(|t| t.status == "stopped"));
    assert!(report.tasks[0].session_id.is_some());
    assert!(dir.join("task-0/process/stdout.jsonl").exists());
    assert_eq!(
        workflow::resume_due(&dir, i64::MAX, 10)
            .await
            .unwrap()
            .state,
        "stopped"
    );
    let report = workflow::resume(&dir, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.tasks.iter().all(|t| t.status == "completed"));
    assert!(!dir.join("stop.request").exists());
    assert!(workflow::merge(&dir).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopping_verification_then_resuming_creates_new_test_evidence() {
    let (root, mut plan) = support::setup("worktree");
    for task in &mut plan.tasks {
        task.test_command[1] = "test-wait".into();
    }
    let dir = root.join("run");
    let workdir = dir.clone();
    let repo = root.join("repo");
    let job = tokio::spawn(async move { workflow::run(plan, &repo, &workdir, 10).await });
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if std::fs::metadata(dir.join("test-0-1/stdout.jsonl")).is_ok_and(|m| m.len() > 0) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    workflow::stop(&dir, "review integration tests").unwrap();
    let report = job.await.unwrap().unwrap();
    assert_eq!(report.state, "stopped");
    assert!(report.tasks.iter().all(|t| t.status == "completed"));
    let report = workflow::resume(&dir, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.verification_attempt, 2);
    assert!(dir.join("test-0-1/stdout.jsonl").exists());
    assert!(dir.join("test-0-2/stdout.jsonl").exists());
    assert!(dir.join("integration-2").exists());
    workflow::approve(&dir, &report.candidate.unwrap().commit).unwrap();
    workflow::merge(&dir).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
