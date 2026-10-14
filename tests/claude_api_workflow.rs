#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn claude_api_error_enters_durable_availability_and_keeps_reviewed_five_hour_fallback() {
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
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    assert!(
        report
            .tasks
            .iter()
            .all(|t| t.status == "waiting_availability")
    );
    let current = report.tasks[0].restriction.as_ref().unwrap().observed_at;
    assert_eq!(
        report.tasks[0].restriction.as_ref().unwrap().available_at,
        None,
        "durations do not supply a reset time"
    );
    let timers = agentconductor::workflow::availability_timer(&report, current);
    assert_eq!(timers.len(), 3);
    assert!(timers.iter().all(|timer| {
        let task = report
            .tasks
            .iter()
            .find(|task| task.id == timer.task)
            .unwrap();
        timer.at == task.restriction.as_ref().unwrap().observed_at + 5 * 3600
    }));
    let reloaded = agentconductor::workflow::load(&root.join("run")).unwrap();
    assert_eq!(
        reloaded.tasks[0].restriction.as_ref().unwrap().observed_at,
        current
    );
    std::fs::remove_dir_all(root).unwrap();
}
