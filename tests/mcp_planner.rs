#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn coordinator_receives_reviewed_source_context_as_untrusted_evidence() {
    let (root, plan) = support::setup("shared");
    let mut agents = plan.agents;
    agents[0].args = vec!["plan".into()];
    agents[0].permission_mode = "read_only".into();
    let context = "## jira:EMM-52599\nSHA-256: evidence\nAcceptance criteria: reject replay.";
    let roles = Default::default();
    let result = agentconductor::planner::generate_from(
        "Implement jira:EMM-52599",
        &agents,
        "mock",
        &root.join("repo"),
        &root.join("planning"),
        &agentconductor::planner::PlanInputs {
            roles: &roles,
            reference: None,
            context: Some(context),
            rules: None,
            phases_root: Some(&root.join("phases")),
        },
    )
    .await
    .unwrap();
    assert!(root.join("phases/plan/analysis.md").exists());
    assert_eq!(result.goal, "Implement jira:EMM-52599");
    let prompt = std::fs::read_to_string(root.join("planning/prompt.txt")).unwrap();
    assert!(prompt.contains("untrusted external content"));
    assert!(prompt.contains("reject replay"));
    assert_eq!(
        std::fs::read_to_string(root.join("planning/source-context.md")).unwrap(),
        context
    );
    std::fs::remove_dir_all(root).unwrap();
}
