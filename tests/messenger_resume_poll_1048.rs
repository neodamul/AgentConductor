//! #1048: a resumed run polls messenger tool decisions too, not only the first start.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{messenger, workflow};

#[tokio::test]
async fn resume_starts_the_decision_poll_for_a_linked_run() {
    let (root, mut plan) = support::setup("shared");
    // A session-capable agent leaves the run resumable, as in tests/native_session.rs.
    let mut profile = serde_json::to_value(&plan.agents[0]).unwrap();
    profile["catalog"] = serde_json::json!({"models":["local/one"],"source":"test","queried_at":1,"cli_version":"fixture"});
    profile["args"] = serde_json::json!(["task", "{model}", "{session_id}"]);
    plan.agents[0] = serde_json::from_value(profile).unwrap();
    let dir = root.join("run");
    let first = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(first.state, "waiting_approval");
    // Not linked when it first ran, so no poll was started.
    assert!(!messenger::inbound::active(&dir));
    // A model change reopens the task, so the run can be stopped and resumed.
    workflow::set_model(&dir, "one.txt", "local/one").unwrap();
    workflow::stop(&dir, "pause").unwrap();
    std::fs::write(
        messenger::link_path(&dir),
        r#"{"workspace":"w","settings":{"decisions":true}}"#,
    )
    .unwrap();
    workflow::resume(&dir, 10).await.unwrap();
    assert!(messenger::inbound::active(&dir));
    std::fs::remove_dir_all(root).unwrap();
}
