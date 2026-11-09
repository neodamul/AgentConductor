#![cfg(unix)]
//! GitHub #637: a refusal sentence printed by a built-in CLI ends the task as permission_denied, not as a plain failure.
use agentconductor::workflow;
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_refusal_sentence_from_a_built_in_cli_is_permission_denied() {
    let (root, mut plan) = support::setup("shared");
    let script = root.join("refusing-agent.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\ncat >/dev/null\necho '{\"type\":\"session\",\"session_id\":\"s1\"}'\necho 'Permission denied by the sandbox'\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    plan.agents[0].executable = script.to_string_lossy().into_owned();
    plan.agents[0].args = vec![];
    plan.agents[0].parser = agentconductor::protocol::Provider::Claude;
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 20)
        .await
        .unwrap();
    let denied = report
        .tasks
        .iter()
        .filter(|t| t.status == "permission_denied")
        .count();
    assert!(
        denied > 0,
        "{:?} {:?}",
        report.state,
        report
            .tasks
            .iter()
            .map(|t| (&t.status, &t.error))
            .collect::<Vec<_>>()
    );
    assert_eq!(report.state, "blocked", "{:?}", report.error);
    std::fs::remove_dir_all(root).unwrap();
}
