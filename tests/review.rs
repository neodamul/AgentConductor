#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{coordinator::Verdict, workflow};

#[tokio::test]
async fn a_stopped_run_refuses_review_without_consuming_a_round_or_starting_a_process() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review".into()];
    plan.agents.push(reviewer);
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    workflow::revise(&dir, "one.txt", "Repair failed task").unwrap();
    workflow::stop(&dir, "inspect before resuming").unwrap();
    assert!(workflow::review(&dir, "coordinator", false).await.is_err());
    assert_eq!(workflow::load(&dir).unwrap().review_round, 0);
    assert!(!dir.join("review-1").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn accepted_cli_review_requires_a_verified_candidate_and_still_cannot_authorize_merge() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review-accept".into()];
    plan.agents.push(reviewer);
    plan.tasks[0].prompt = "fail".into();
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert!(workflow::review(&dir, "coordinator", true).await.is_err());
    workflow::revise(&dir, "one.txt", "Repair failed task").unwrap();
    workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("review")
        .arg(&dir)
        .arg("coordinator")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["proposal"]["verdict"], "accept");
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    assert!(workflow::merge(&dir).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn human_feedback_invalidates_a_saved_proposal_even_if_its_evidence_files_are_unchanged() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review".into()];
    plan.agents.push(reviewer);
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let proposal = workflow::review(&dir, "coordinator", false).await.unwrap();
    workflow::revise(&dir, "one.txt", "Use version two").unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&proposal).unwrap());
    assert!(workflow::apply_review(&dir, &digest).is_err());
    assert_eq!(
        workflow::load(&dir).unwrap().tasks[0].feedback.as_deref(),
        Some("Use version two")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stopping_a_review_preserves_its_logs_and_does_not_apply_unfinished_feedback() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review-slow".into()];
    plan.agents.push(reviewer);
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let run_dir = dir.clone();
    let job = tokio::spawn(async move { workflow::review(&run_dir, "coordinator", true).await });
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while !std::fs::metadata(dir.join("review-1/process/stdout.jsonl"))
            .is_ok_and(|m| m.len() > 0)
        {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    workflow::stop(&dir, "inspect reviewer").unwrap();
    assert!(job.await.unwrap().is_err());
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "stopped");
    assert!(report.tasks.iter().all(|t| t.feedback.is_none()));
    assert!(dir.join("review-1/process/owner.json").exists());
    assert!(workflow::merge(&dir).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn coordinator_write_violation_revokes_candidate_and_does_not_rebaseline_edits() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review-write".into()];
    plan.agents.push(reviewer);
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert!(workflow::review(&dir, "coordinator", true).await.is_err());
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "failed");
    assert!(report.candidate.is_none());
    assert!(workflow::resume_due(&dir, i64::MAX, 10).await.is_err());
    assert!(workflow::merge(&dir).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn applying_a_review_uses_the_exact_saved_proposal_without_another_model_call() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review".into()];
    plan.agents.push(reviewer);
    plan.tasks[0].prompt = "fail".into();
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let proposal = workflow::review(&dir, "coordinator", false).await.unwrap();
    assert!(workflow::load(&dir).unwrap().tasks[0].feedback.is_none());
    assert!(workflow::apply_review(&dir, "wrong").is_err());
    let digest = workflow::digest(&serde_json::to_vec(&proposal).unwrap());
    let applied = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("apply-review")
        .arg(&dir)
        .arg(&digest)
        .output()
        .unwrap();
    assert!(
        applied.status.success(),
        "{}",
        String::from_utf8_lossy(&applied.stderr)
    );
    assert_eq!(workflow::load(&dir).unwrap().state, "ready");
    assert_eq!(workflow::load(&dir).unwrap().review_round, 1);
    assert!(workflow::apply_review(&dir, &digest).is_err());
    assert!(!dir.join("review-2").exists());
    let report = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(workflow::merge(&dir).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn edited_evidence_invalidates_a_saved_review_before_any_feedback_is_applied() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review".into()];
    plan.agents.push(reviewer);
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let proposal = workflow::review(&dir, "coordinator", false).await.unwrap();
    std::fs::write(
        dir.join("task-0/process/stderr.log"),
        "modified after review",
    )
    .unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&proposal).unwrap());
    assert!(workflow::apply_review(&dir, &digest).is_err());
    assert!(
        workflow::load(&dir)
            .unwrap()
            .tasks
            .iter()
            .all(|t| t.feedback.is_none())
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn structured_review_repairs_a_failure_but_never_approves_merge() {
    let (root, mut plan) = support::setup("worktree");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review".into()];
    plan.agents.push(reviewer);
    plan.tasks[0].prompt = "fail".into();
    plan.tasks[1].depends_on = vec![plan.tasks[0].id.clone()];
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let proposal = workflow::review(&dir, "coordinator", true).await.unwrap();
    assert_eq!(proposal.verdict, Verdict::Revise);
    assert_eq!(workflow::load(&dir).unwrap().state, "ready");
    assert!(dir.join("review-1/process/stdout.jsonl").exists());
    let report = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(workflow::merge(&dir).is_err());
    workflow::approve(&dir, &report.candidate.unwrap().commit).unwrap();
    workflow::merge(&dir).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn invalid_review_preserves_original_assignments_and_scopes() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review-unknown".into()];
    plan.agents.push(reviewer);
    let initial = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert!(workflow::review(&dir, "coordinator", true).await.is_err());
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, initial.state);
    assert_eq!(report.tasks[0].attempt, 1);
    assert!(report.tasks.iter().all(|t| t.feedback.is_none()));
    assert_eq!(report.plan.tasks[0].paths, initial.plan.tasks[0].paths);
    std::fs::remove_dir_all(root).unwrap();
}
