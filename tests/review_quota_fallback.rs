#![cfg(unix)]
use agentconductor::{plan::Fallback, workflow};

#[path = "support/workflow.rs"]
mod support;

fn plan(mode: &str) -> (std::path::PathBuf, agentconductor::plan::Plan) {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec![mode.into()];
    plan.agents[0].quota_group = Some("paid-account".into());

    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.args = vec!["review".into()];
    coordinator.permission_mode = "read_only".into();
    coordinator.fallback = Some(Fallback {
        agent: "company".into(),
        model: Some("local/one".into()),
    });

    let mut company = coordinator.clone();
    company.id = "company".into();
    company.quota_group = Some("company-account".into());
    company.fallback = None;
    company.catalog = Some(agentconductor::plan::ModelCatalog {
        models: vec!["local/one".into()],
        source: "fixture".into(),
        queried_at: 1,
        cli_version: "fixture".into(),
        origin: Default::default(),
    });
    company.execution_options = vec![agentconductor::plan::ExecutionOption {
        name: "model".into(),
        args: vec!["--model".into(), "{value}".into()],
        values: vec![],
        default: None,
        model_values: Default::default(),
    }];
    company
        .environment
        .insert("AC_EXPECTED_MODEL".into(), "local/one".into());
    plan.agents.extend([coordinator, company]);
    plan.validate().unwrap();
    (root, plan)
}

#[tokio::test]
async fn quota_restricted_coordinator_uses_registered_fallback_after_existing_deadline() {
    let (root, plan) = plan("quota-long");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");

    let proposal = workflow::review(&run, "coordinator", false).await.unwrap();
    assert_eq!(
        proposal.verdict,
        agentconductor::coordinator::Verdict::Revise
    );
    assert_eq!(workflow::load(&run).unwrap().review_round, 1);
    let connection = rusqlite::Connection::open(run.join("state.sqlite")).unwrap();
    let payload: String = connection
        .query_row(
            "SELECT payload FROM audit WHERE event='review_start' ORDER BY seq DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let event: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(event["requested_coordinator"], "coordinator");
    assert_eq!(event["coordinator"], "company");
    assert_eq!(event["coordinator_model"], "local/one");
    assert_eq!(event["fallback_from"], "coordinator");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn coordinator_fallback_before_the_existing_deadline_starts_no_review_process() {
    let (root, plan) = plan("quota");
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    assert!(workflow::review(&run, "coordinator", false).await.is_err());
    assert!(!run.join("review-1").exists());
    std::fs::remove_dir_all(root).unwrap();
}
