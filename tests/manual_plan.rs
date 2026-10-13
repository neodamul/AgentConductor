#![cfg(unix)]
//! R-37 / DD-45 / QAS-29: the operator writes the plan without any coordinator; the draft
//! uses the same validation, exact digest approval and repository-base binding.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}
fn open(root: &Path, plan: &agentconductor::plan::Plan) -> Startup {
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    // The registered agent cannot plan read-only; a draft must not need it to.
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":root.join("repo"),"goal":"Create three scoped files by hand","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    startup
}
fn git(root: &Path, args: &[&str]) {
    assert!(
        std::process::Command::new("git")
            .arg("-C")
            .arg(root.join("repo"))
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn manual_draft_needs_no_coordinator_and_runs_only_after_complete_exact_approval() {
    let (root, plan) = support::setup("shared");
    let startup = open(&root, &plan);
    mutate(&startup, serde_json::json!({"kind":"draft"})).unwrap();
    let draft = snapshot(&startup);
    assert!(draft["job"].is_null(), "a draft launches no process");
    assert_eq!(draft["proposal"]["phase"], "ready");
    let tasks = draft["proposal"]["plan"]["tasks"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(tasks.len(), 3);
    assert!(
        tasks
            .iter()
            .all(|t| t["agent"] == "mock" && t["prompt"] == "")
    );
    let start = |state: &serde_json::Value| {
        mutate(
            &startup,
            serde_json::json!({"kind":"start","digest":state["proposal"]["digest"]}),
        )
    };
    assert!(
        start(&draft).is_err(),
        "an unfilled draft is never approvable"
    );
    let edit = |tasks: serde_json::Value| {
        mutate(
            &startup,
            serde_json::json!({"kind":"edit","tasks":tasks,"workspace_mode":"shared"}),
        )
    };
    assert!(edit(serde_json::json!(tasks)).is_err());
    let mut written = serde_json::to_value(&plan.tasks).unwrap();
    written[1]["paths"] = written[0]["paths"].clone();
    assert!(edit(written).is_err(), "overlapping scopes are refused");
    edit(serde_json::to_value(&plan.tasks).unwrap()).unwrap();
    let reviewed = snapshot(&startup);
    let accepted = start(&reviewed).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while snapshot(&startup)["job"]["state"] == "running" {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let report =
        agentconductor::workflow::load(Path::new(accepted["run"].as_str().unwrap())).unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.tests.len(), 3);
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manual_draft_is_bound_to_the_repository_base_and_a_clean_tree() {
    let (root, plan) = support::setup("shared");
    let startup = open(&root, &plan);
    std::fs::write(root.join("repo/dirty.txt"), "uncommitted").unwrap();
    assert!(mutate(&startup, serde_json::json!({"kind":"draft"})).is_err());
    git(&root, &["add", "dirty.txt"]);
    git(&root, &["commit", "-m", "dirty"]);
    mutate(&startup, serde_json::json!({"kind":"draft"})).unwrap();
    mutate(
        &startup,
        serde_json::json!({"kind":"edit","tasks":plan.tasks,"workspace_mode":"shared"}),
    )
    .unwrap();
    std::fs::write(root.join("repo/later.txt"), "moved").unwrap();
    git(&root, &["add", "later.txt"]);
    git(&root, &["commit", "-m", "later"]);
    let reviewed = snapshot(&startup);
    let error = mutate(
        &startup,
        serde_json::json!({"kind":"start","digest":reviewed["proposal"]["digest"]}),
    )
    .unwrap_err();
    assert!(
        error.contains(agentconductor::ui::messages::text(
            "native.repository_changed_since_planning_generate_a_new_plan"
        )),
        "{error}"
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
