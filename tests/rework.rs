#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;

#[tokio::test]
async fn feedback_repairs_failure_releases_dependents_and_keeps_unaffected_completed_work() {
    let (root, mut plan) = support::setup("worktree");
    let dir = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    plan.tasks[1].depends_on = vec![plan.tasks[0].id.clone()];
    let initial = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(initial.tasks[0].status, "failed");
    assert_eq!(initial.tasks[1].status, "blocked");
    workflow::revise(&dir, "one.txt", "Repair failed task").unwrap();
    let report = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.tasks[0].attempt, 2);
    assert_eq!(report.tasks[1].attempt, 1);
    assert_eq!(report.tasks[2].attempt, 1);
    assert!(dir.join("task-0/process/stderr.log").exists());
    assert_eq!(report.plan.tasks[0].prompt, "fail");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn review_feedback_refreshes_transitive_dependencies_and_revokes_old_approval() {
    for mode in ["shared", "worktree"] {
        let (root, mut plan) = support::setup(mode);
        plan.agents[0].args = vec!["dependency".into()];
        plan.tasks[1].depends_on = vec![plan.tasks[0].id.clone()];
        plan.tasks[2].depends_on = vec![plan.tasks[1].id.clone()];
        for task in &mut plan.tasks {
            task.test_command[1] = "dependency-test".into();
        }
        let dir = root.join("run");
        let initial = workflow::run(plan, &root.join("repo"), &dir, 10)
            .await
            .unwrap();
        assert_eq!(initial.state, "waiting_approval");
        let old = initial.candidate.as_ref().unwrap().commit.clone();
        workflow::approve(&dir, &old).unwrap();
        let revised_command = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .arg("revise")
            .arg(&dir)
            .arg("one.txt")
            .arg("Use version two")
            .output()
            .unwrap();
        assert!(
            revised_command.status.success(),
            "{}",
            String::from_utf8_lossy(&revised_command.stderr)
        );
        assert!(workflow::merge(&dir).is_err());
        let revised = workflow::load(&dir).unwrap();
        assert!(revised.tasks.iter().all(|t| t.status == "pending"));
        assert!(revised.tests.is_empty());
        let final_report = workflow::resume_due(&dir, i64::MAX, 10).await.unwrap();
        assert_eq!(
            final_report.state, "waiting_approval",
            "{:?}",
            final_report.error
        );
        assert!(final_report.tasks.iter().all(|t| t.attempt == 2));
        assert_eq!(
            std::fs::read_to_string(final_report.verification_cwd.join("three.txt")).unwrap(),
            "documented observed v2\n"
        );
        if mode == "worktree" {
            assert_ne!(initial.tasks[1].cwd, final_report.tasks[1].cwd);
            assert_eq!(
                std::fs::read_to_string(initial.tasks[1].cwd.join("two.txt")).unwrap(),
                "observed v1\n"
            );
            assert_ne!(
                initial.tasks[1].session_id,
                final_report.tasks[1].session_id
            );
        }
        assert!(dir.join("task-0/process/stdout.jsonl").exists());
        assert!(workflow::approve(&dir, &old).is_err());
        workflow::approve(&dir, &final_report.candidate.unwrap().commit).unwrap();
        workflow::merge(&dir).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn feedback_is_bounded_and_cannot_revise_a_merged_run() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert!(workflow::revise(&dir, "one.txt", "").is_err());
    assert!(workflow::revise(&dir, "one.txt", &"x".repeat(64 * 1024 + 1)).is_err());
    assert!(workflow::revise(&dir, "missing", "feedback").is_err());
    workflow::approve(&dir, &report.candidate.unwrap().commit).unwrap();
    workflow::merge(&dir).unwrap();
    assert!(workflow::revise(&dir, "one.txt", "feedback").is_err());
    std::fs::remove_dir_all(root).unwrap();
}
