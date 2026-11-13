#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn claude_api_error_keeps_durable_restriction_and_uses_reviewed_fallback_immediately() {
    let (root, mut plan) = support::setup("shared");
    let mut backup = plan.agents[0].clone();
    backup.id = "backup".into();
    plan.agents[0].parser = agentconductor::protocol::Provider::Claude;
    plan.agents[0].executable = if cfg!(windows) { "python" } else { "python3" }.into();
    plan.agents[0].args = vec![
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/claude_api_worker.py"
        )
        .into(),
    ];
    plan.agents[0].permission_mode = "read_only".into();
    plan.agents[0].fallback = Some(agentconductor::plan::Fallback {
        agent: "backup".into(),
        model: None,
    });
    plan.agents.push(backup);
    let quota_key = plan.agents[0].quota_key().to_string();
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.tasks.iter().all(|t| t.status == "completed"));
    assert!(report.plan.tasks.iter().all(|t| t.agent == "backup"));
    let restriction = &report.account_restrictions[&quota_key];
    let current = restriction.observed_at;
    assert_eq!(
        restriction.available_at, None,
        "durations do not supply a reset time"
    );
    assert!(agentconductor::workflow::availability_timer(&report, current).is_empty());
    let reloaded = agentconductor::workflow::load(&root.join("run")).unwrap();
    assert_eq!(
        reloaded.account_restrictions[&quota_key].observed_at,
        current
    );
    std::fs::remove_dir_all(root).unwrap();
}
