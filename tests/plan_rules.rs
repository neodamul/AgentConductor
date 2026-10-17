#![cfg(unix)]
//! R-59 / DD-65 / QAS-50: editable plan rules reach the planning coordinator.
use agentconductor::plan_rules;
use std::process::Command;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn global_rules_are_created_once_and_workspace_text_overrides_them() {
    let (root, _) = support::setup("shared");
    let home = root.join("home");
    // Single test in this binary touches HOME.
    unsafe { std::env::set_var("HOME", &home) };
    let path = plan_rules::global_path().unwrap();
    assert_eq!(path, home.join(".agentconductor/plan-rules.md"));
    let rules = plan_rules::effective(None).unwrap();
    for topic in ["analysis", "phase", "verification", "300", "English", "TDD"] {
        assert!(rules.contains(topic), "default rules mention {topic}");
    }
    assert!(
        !rules.contains("500 lines"),
        "the old file limit is removed"
    );
    std::fs::write(&path, "# Team rules\n- Keep files under 300 lines\n").unwrap();
    assert!(
        plan_rules::effective(None).unwrap().contains("300 lines"),
        "edits are kept"
    );
    assert_eq!(
        plan_rules::effective(Some("Workspace only")).unwrap(),
        "Workspace only"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn planning_prompt_carries_the_rules_file() {
    let (root, plan) = support::setup("shared");
    let mut planner = plan.agents[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan".into()];
    planner.permission_mode = "read_only".into();
    let profiles = root.join("profiles.json");
    std::fs::write(
        &profiles,
        serde_json::to_vec(&vec![plan.agents[0].clone(), planner]).unwrap(),
    )
    .unwrap();
    let rules = root.join("rules.md");
    std::fs::write(&rules, "- Always start with an analysis section\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("plan")
        .arg(&profiles)
        .arg("planner")
        .arg(root.join("repo"))
        .arg(root.join("ruled"))
        .arg("Goal")
        .arg("--rules")
        .arg(&rules)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let prompt = std::fs::read_to_string(root.join("ruled/prompt.txt")).unwrap();
    assert!(
        prompt.contains("Plan rules") && prompt.contains("Always start with an analysis section"),
        "{prompt}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
