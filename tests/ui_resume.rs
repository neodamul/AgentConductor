use agentconductor::workflow;
use std::fs;
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_newer_stop_cannot_be_cleared_by_an_already_accepted_resume() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-known".into()];
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    workflow::stop(&dir, "First review").unwrap();
    let old = workflow::digest(&fs::read(dir.join("stop.request")).unwrap());
    workflow::stop(&dir, "Keep this new stop").unwrap();
    let resumed = workflow::resume_reviewed_stop(&dir, 5, &old).await.unwrap();
    assert_eq!(resumed.state, "stopped");
    assert_eq!(resumed.tasks[0].attempt, 1);
    let marker: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("stop.request")).unwrap()).unwrap();
    assert_eq!(marker["reason"], "Keep this new stop");
    fs::remove_dir_all(root).unwrap();
}
