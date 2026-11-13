//! #925: unknown-reset limits must not park the first task behind later ready work.
use agentconductor::{plan::Fallback, workflow};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn fresh_unknown_limit_continues_on_fallback_with_independent_and_dependent_work() {
    for mode in ["shared", "worktree"] {
        let (root, mut plan) = support::setup(mode);
        let mut backup = plan.agents[0].clone();
        backup.id = "company".into();
        backup.quota_group = Some("company-account".into());
        backup.args = vec!["task-preserve".into()];
        let mut later = backup.clone();
        later.id = "later".into();
        later.args = vec!["task".into()];
        plan.agents[0].args = vec!["quota-partial".into()];
        plan.agents[0].quota_group = Some("paid-account".into());
        plan.agents[0].fallback = Some(Fallback {
            agent: "company".into(),
            model: None,
        });
        plan.agents.extend([backup, later]);
        plan.tasks[1].agent = "later".into();
        plan.tasks[2].agent = "later".into();
        plan.tasks[2].depends_on = vec![plan.tasks[0].id.clone()];
        let run = root.join("run");
        let report = workflow::run(plan, &root.join("repo"), &run, 5)
            .await
            .unwrap();
        assert_eq!(
            report.state, "waiting_approval",
            "{mode}: {:?}",
            report.error
        );
        assert_eq!(report.plan.tasks[0].agent, "company");
        assert_eq!(report.tasks[0].attempt, 2);
        assert!(report.tasks.iter().all(|task| task.status == "completed"));
        assert!(report.tasks[2].started_ms >= report.tasks[0].finished_ms);
        assert_eq!(
            std::fs::read_to_string(report.tasks[0].cwd.join("one.txt")).unwrap(),
            "partial\nimplemented\n"
        );
        assert_eq!(report.tests.len(), 3);
        assert!(
            report.account_restrictions["paid-account"]
                .available_at
                .is_none()
        );
        assert!(
            workflow::merge(&run).is_err(),
            "fallback does not grant merge approval"
        );
        workflow::release_run_worktrees(&run);
        std::fs::remove_dir_all(root).unwrap();
    }
}
