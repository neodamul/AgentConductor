#![cfg(unix)]
//! R-56 / DD-62: the coordinator returns an analysis and ordered phases; the slug tree
//! records them and the first phase becomes the reviewable run plan.
use agentconductor::slug_progress::progress;
use std::process::Command;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn phased_planning_writes_the_slug_tree_and_the_first_phase_plan() {
    let (root, plan) = support::setup("shared");
    let mut planner = plan.agents[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan-phases".into()];
    planner.permission_mode = "read_only".into();
    let profiles = root.join("profiles.json");
    std::fs::write(
        &profiles,
        serde_json::to_vec(&vec![plan.agents[0].clone(), planner]).unwrap(),
    )
    .unwrap();
    let slug = root.join("slug");
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("plan")
        .arg(&profiles)
        .arg("planner")
        .arg(root.join("repo"))
        .arg(root.join("planning"))
        .arg("Phased goal")
        .arg("--phases")
        .arg(&slug)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let prompt = std::fs::read_to_string(root.join("planning/prompt.txt")).unwrap();
    assert!(prompt.contains("analysis") && prompt.contains("\"phases\""));
    assert_eq!(
        std::fs::read_to_string(slug.join("plan/analysis.md")).unwrap(),
        "# Analysis\nCurrent state"
    );
    let state = progress(&slug);
    assert_eq!(state["current_phase"], "01-core");
    assert_eq!(state["phases"].as_array().unwrap().len(), 2);
    let first: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("planning/plan.json")).unwrap()).unwrap();
    let ids: Vec<_> = first["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        ["a", "b"],
        "only the first phase is reviewed for execution"
    );
    std::fs::remove_dir_all(root).unwrap();
}
