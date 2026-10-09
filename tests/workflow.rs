#![cfg(unix)]
use agentconductor::workflow;

#[path = "support/workflow.rs"]
mod support;
use support::{fixture, setup};

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
