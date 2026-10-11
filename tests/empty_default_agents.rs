#![cfg(unix)]
//! R-66 / DD-72 / QAS-59: a new workspace has no agents until the operator adds them.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn new_workspace_starts_empty_with_templates_available() {
    let (root, plan) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(
        snapshot["config"]["profiles"]
            .as_array()
            .unwrap()
            .is_empty(),
        "no pre-registered agents"
    );
    assert_eq!(snapshot["config"]["coordinator"], "");
    let templates: Vec<_> = snapshot["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    assert!(templates.contains(&"claude".into()) && templates.contains(&"codex".into()));
    let config = serde_json::json!({"profiles":[],"repository":root.join("repo"),"goal":"g","coordinator":"","workspace_mode":"shared"});
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":snapshot["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .expect("an empty workspace saves");
    let revision = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"].clone();
    assert!(
        startup
            .dispatch(
                serde_json::json!({"kind":"draft","revision":revision})
                    .to_string()
                    .as_bytes()
            )
            .is_err(),
        "no plan without agents"
    );
    // Tests add their agents first, then work as before.
    let config = serde_json::json!({"profiles":plan.agents,"repository":root.join("repo"),"goal":"g","coordinator":"mock","workspace_mode":"shared"});
    let revision = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"].clone();
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":revision,"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let revision = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"].clone();
    startup
        .dispatch(
            serde_json::json!({"kind":"draft","revision":revision})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
