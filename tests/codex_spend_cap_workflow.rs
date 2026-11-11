#![cfg(feature = "test-fixtures")]

#[path = "support/workflow.rs"]
mod support;

use agentconductor::{plan::Fallback, protocol::Provider, workflow};

#[tokio::test]
async fn codex_spend_cap_persists_restriction_and_selects_reviewed_fallback() {
    let (root, mut plan) = support::setup("shared");
    let fixture = support::fixture().to_string_lossy().into_owned();
    let mut codex = plan.agents.remove(0);
    codex.id = "codex".into();
    codex.parser = Provider::Codex;
    codex.quota_group = Some("codex-enterprise".into());
    codex.executable = fixture.clone();
    codex.args = vec!["spend-cap".into()];
    codex.permission_mode = "read_only".into();
    codex.fallback = Some(Fallback {
        agent: "backup".into(),
        model: None,
    });
    let mut backup = codex.clone();
    backup.id = "backup".into();
    backup.parser = Provider::Claude;
    backup.quota_group = Some("backup".into());
    backup.args = vec!["task".into()];
    backup.fallback = None;
    plan.agents = vec![codex, backup];
    plan.tasks.truncate(1);
    plan.tasks[0].agent = "codex".into();
    plan.tasks[0].test_command = vec![fixture, "test-pass".into()];

    let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
        .await
        .unwrap();

    assert_eq!(report.state, "waiting_availability");
    assert_eq!(report.tasks[0].status, "waiting_availability");
    assert!(report.tasks[0].restriction.is_some());
    let actions = workflow::availability_timer(
        &report,
        report.tasks[0].restriction.as_ref().unwrap().observed_at,
    );
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].kind, "fallback");

    let resumed = workflow::resume_availability_due(
        &root.join("run"),
        report.tasks[0].restriction.as_ref().unwrap().observed_at + workflow::FALLBACK_WAIT_SECONDS,
        10,
    )
    .await
    .unwrap();
    assert_eq!(resumed.plan.tasks[0].agent, "backup");
    assert_eq!(
        resumed.tasks[0].status, "completed",
        "task error: {:?}",
        resumed.tasks[0].error
    );
    std::fs::remove_dir_all(root).unwrap();
}
