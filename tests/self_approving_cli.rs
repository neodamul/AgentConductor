//! A custom CLI that approves its own tools (for example aipro set to auto-approve tools) can
//! run once the operator marks it so; unmarked custom CLIs still cannot.
use agentconductor::{ui::setup::Startup, workflow};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn a_marked_custom_cli_keeps_its_permission_when_saved() {
    let root = std::env::temp_dir().join(format!(
        "ac-self-approve-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut config = state["config"].clone();
    config["profiles"] = json!([{
        "id": "aipro", "executable": "aipro", "args": [], "parser": "custom",
        "permission_mode": "operator_auto_approve"
    }]);
    config["coordinator"] = "aipro".into();
    startup
        .dispatch(
            json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let saved = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        saved["config"]["profiles"][0]["permission_mode"],
        "operator_auto_approve"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn only_a_marked_custom_cli_may_run() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].permission_mode = "unverified".into();
    let refused = workflow::run(plan.clone(), &root.join("repo"), &root.join("run-a"), 10).await;
    assert!(
        refused
            .err()
            .is_some_and(|e| e.contains("mark that in its agent settings"))
    );
    plan.agents[0].permission_mode = "operator_auto_approve".into();
    let report = workflow::run(plan, &root.join("repo"), &root.join("run-b"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    std::fs::remove_dir_all(root).unwrap();
}
