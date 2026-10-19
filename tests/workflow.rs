#![cfg(unix)]
use agentconductor::workflow;

#[path = "support/workflow.rs"]
mod support;
use support::{fixture, setup, timeout_once_fixture};

#[tokio::test]
async fn shared_and_worktree_modes_run_parallel_then_require_exact_human_approval() {
    for mode in ["shared", "worktree"] {
        let (root, plan) = setup(mode);
        let repo = root.join("repo");
        let run = root.join("run");
        let report = workflow::run(plan, &repo, &run, 10).await.unwrap();
        assert_eq!(report.state, "waiting_approval");
        assert!(report.tasks.iter().all(|t| t.status == "completed"));
        let base = report.base.clone();
        let candidate = report.candidate.as_ref().unwrap().commit.clone();
        let git = agentconductor::git::Repository::open(&repo).unwrap();
        assert_eq!(git.head().unwrap(), base);
        assert!(workflow::merge(&run).is_err());
        workflow::approve(&run, &candidate).unwrap();
        workflow::merge(&run).unwrap();
        assert_eq!(git.head().unwrap(), candidate);
        git.clean().unwrap();
        for name in ["one.txt", "two.txt", "three.txt"] {
            assert!(repo.join(name).exists());
        }
        // Each task records its start before the others complete.
        let ranges: Vec<_> = report
            .tasks
            .iter()
            .map(|t| (t.started_ms.unwrap(), t.finished_ms.unwrap()))
            .collect();
        assert!(
            ranges.iter().map(|p| p.0).max().unwrap() < ranges.iter().map(|p| p.1).min().unwrap()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn custom_cli_event_contract_completes_the_reviewable_workflow() {
    use agentconductor::protocol::{EventKind, Provider};

    let (root, mut plan) = setup("shared");
    plan.agents[0].parser = Provider::Custom;
    plan.agents[0].args = vec!["custom-task".into()];
    plan.agents[0]
        .event_types
        .insert("vendor.session".into(), EventKind::Session);
    plan.agents[0]
        .event_types
        .insert("vendor.done".into(), EventKind::Result);
    plan.agents[0].session_id_pointer = Some("/meta/id".into());

    let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.tasks.iter().all(|task| {
        task.status == "completed"
            && task
                .session_id
                .as_deref()
                .is_some_and(|id| id.starts_with("vendor-"))
    }));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn changed_evidence_and_changed_candidate_block_merge() {
    let (root, plan) = setup("shared");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    workflow::approve(&run, &report.candidate.unwrap().commit).unwrap();
    std::fs::write(root.join("repo/one.txt"), "changed after review").unwrap();
    assert!(workflow::merge(&run).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn failure_blocks_dependents_and_does_not_create_merge_candidate() {
    let (root, mut plan) = setup("worktree");
    let run = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    plan.tasks[1].depends_on = vec![plan.tasks[0].id.clone()];
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    assert_eq!(report.tasks[1].status, "blocked");
    assert!(report.candidate.is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn timed_out_worker_waits_for_explicit_resume_and_keeps_timeout_evidence() {
    let (root, mut plan) = setup("worktree");
    let run = root.join("run");
    let markers = root.join("markers");
    std::fs::create_dir_all(&markers).unwrap();
    plan.agents[0].executable = timeout_once_fixture().to_string_lossy().into_owned();
    plan.agents[0].args.clear();
    plan.agents[0].environment.insert(
        "MOCK_MARKERS".into(),
        markers.to_string_lossy().into_owned(),
    );

    let timed_out = workflow::run(plan, &root.join("repo"), &run, 1)
        .await
        .unwrap();

    assert_eq!(timed_out.state, "blocked");
    assert_eq!(timed_out.tasks[0].status, "blocked");
    assert_eq!(timed_out.tasks[0].attempt, 1);
    let first: serde_json::Value = serde_json::from_slice(
        &std::fs::read(timed_out.tasks[0].artifacts.join("process/owner.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(first["timed_out"], true);

    let report = workflow::resume_due(&run, (now_ms() / 1000) as i64, 1)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.tasks[0].status, "completed");
    assert_eq!(report.tasks[0].attempt, 2);
    std::fs::remove_dir_all(root).unwrap();
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[tokio::test]
async fn failing_tests_and_scope_escape_cannot_be_approved() {
    for error in ["test", "scope"] {
        let (root, mut plan) = setup("shared");
        let run = root.join("run");
        if error == "test" {
            plan.tasks[0].test_command = vec![fixture().to_str().unwrap().into(), "fail".into()];
        } else {
            plan.tasks[0].prompt = "outside.txt".into();
        }
        let report = workflow::run(plan, &root.join("repo"), &run, 10)
            .await
            .unwrap();
        assert_eq!(report.state, "failed");
        assert!(workflow::approve(&run, "invalid").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn target_branch_advanced_after_approval_blocks_merge() {
    let (root, plan) = setup("worktree");
    let repo = root.join("repo");
    let run = root.join("run");
    let report = workflow::run(plan, &repo, &run, 10).await.unwrap();
    workflow::approve(&run, &report.candidate.unwrap().commit).unwrap();
    let git = agentconductor::git::Repository::open(&repo).unwrap();
    std::fs::write(repo.join("late.txt"), "someone else committed").unwrap();
    git.command(&["add", "late.txt"]).unwrap();
    git.command(&["commit", "-m", "late"]).unwrap();
    let moved = git.head().unwrap();
    let error = workflow::merge(&run).unwrap_err();
    assert!(error.contains("target branch changed"), "{error}");
    assert_eq!(git.head().unwrap(), moved);
    std::fs::remove_dir_all(root).unwrap();
}
