//! GitHub #1024: the operator sets or clears a task's effort on an editable run; it must be verified again.
#![cfg(unix)]
use agentconductor::workflow;
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_task_effort_is_set_cleared_and_validated() {
    let (root, mut plan) = support::setup("shared");
    let mut profile = serde_json::to_value(&plan.agents[0]).unwrap();
    profile["execution_options"] = serde_json::json!([{"name":"effort","args":["--effort","{value}"],"values":["low","medium","high"]}]);
    plan.agents[0] = serde_json::from_value(profile).unwrap();
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    let effort = || {
        workflow::load(&dir).unwrap().plan.tasks[0]
            .execution_options
            .get("effort")
            .cloned()
    };
    workflow::set_effort(&dir, "one.txt", Some("high")).unwrap();
    assert_eq!(effort().as_deref(), Some("high"));
    assert!(workflow::set_effort(&dir, "one.txt", Some("turbo")).is_err());
    assert_eq!(
        effort().as_deref(),
        Some("high"),
        "a refused effort changes nothing"
    );
    workflow::set_effort(&dir, "one.txt", None).unwrap();
    assert_eq!(effort(), None, "None returns to the agent default");
    std::fs::remove_dir_all(root).unwrap();
}
