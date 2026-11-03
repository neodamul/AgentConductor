//! DD-141 (amended by the user, 2026-10-14): a custom CLI such as aipro runs as approving its
//! own tools by default, without an operator checkbox.
use agentconductor::{ui::setup::Startup, workflow};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn a_custom_cli_is_saved_as_approving_its_own_tools() {
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
        "permission_mode": "unverified"
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
async fn a_derived_custom_cli_runs_and_a_raw_unverified_plan_is_explained() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].permission_mode = "unverified".into();
    let refused = workflow::run(plan.clone(), &root.join("repo"), &root.join("run-a"), 10).await;
    assert!(
        refused
            .err()
            .is_some_and(|e| e.contains("save the agent list once"))
    );
    plan.agents[0].permission_mode = "operator_auto_approve".into();
    let report = workflow::run(plan, &root.join("repo"), &root.join("run-b"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    std::fs::remove_dir_all(root).unwrap();
}
