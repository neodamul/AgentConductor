#![cfg(unix)]
//! GitHub #210: the plan review shows what kind of work each task is and the effort it will run with, and keeps the
//! type when the operator edits the plan; the roles table says which efforts a model honors.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = include_str!("../src/ui/startup.js");
const EN: &str = include_str!("../src/ui/messages/en.json");
const KO: &str = include_str!("../src/ui/messages/ko.json");

#[test]
fn the_review_snapshot_carries_the_resolved_profile_of_each_task() {
    let (root, plan) = support::setup("shared");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let mut profiles = serde_json::to_value(&plan.agents).unwrap();
    profiles[0]["execution_options"] = serde_json::json!([{"name":"effort","args":["--effort","{value}"],"values":["low","medium","high"]}]);
    let snapshot = |s: &Startup| s.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mutate = |s: &Startup, mut p: serde_json::Value| {
        p["revision"] = snapshot(s)["revision"].clone();
        s.dispatch(p.to_string().as_bytes()).unwrap()
    };
    mutate(
        &startup,
        serde_json::json!({"kind":"save","config":{"profiles":profiles,"repository":root.join("repo"),"goal":"g","coordinator":"mock","workspace_mode":"shared"}}),
    );
    mutate(&startup, serde_json::json!({"kind":"draft"}));
    let state = snapshot(&startup);
    let shown = state["proposal"]["task_profiles"].as_array().unwrap();
    assert_eq!(
        shown.len(),
        state["proposal"]["plan"]["tasks"].as_array().unwrap().len()
    );
    assert_eq!(
        shown[0]["task_type"], "code",
        "a task without a type is code"
    );
    assert_eq!(shown[0]["effort"], "high");
    assert_eq!(shown[0]["warnings"], serde_json::json!([]));
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_page_shows_the_profile_keeps_the_type_and_hints_the_efforts() {
    for needle in [
        "proposal.task_profiles",
        "dataset.taskType",
        "task_type:card.dataset.taskType",
        "task-profile",
        "modelHint(",
    ] {
        assert!(SCRIPT.contains(needle), "startup.js lacks {needle}");
    }
    for catalog in [EN, KO] {
        let messages: serde_json::Value = serde_json::from_str(catalog).unwrap();
        for id in [
            "startup.task_profile",
            "startup.task_type_extract",
            "startup.task_type_summarize",
            "startup.task_type_analyze",
            "startup.task_type_code",
            "startup.task_type_agentic",
            "startup.task_type_creative",
            "startup.task_warning_effort_unsupported",
            "startup.task_warning_model_not_suited",
            "startup.model_hint",
        ] {
            assert!(messages[id].as_str().is_some_and(|m| !m.is_empty()), "{id}");
        }
    }
}

#[test]
fn the_page_helpers_describe_the_profile_and_the_model() {
    let result = std::process::Command::new("node")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/task_profile_runtime.cjs")
        .output()
        .expect("Node runs UI behavior tests");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
