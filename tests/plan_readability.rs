#![cfg(unix)]
//! R-46 / DD-54 / QAS-39: sectioned instructions in the goal's language; roomy editor.
use std::process::Command;
#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = include_str!("../src/ui/startup.js");

#[test]
fn planning_prompt_requires_sectioned_instructions_in_the_goal_language() {
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
    let dir = root.join("planning");
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("plan")
        .arg(&profiles)
        .arg("planner")
        .arg(root.join("repo"))
        .arg(&dir)
        .arg("헬스체크 기능을 추가한다")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let prompt = std::fs::read_to_string(dir.join("prompt.txt")).unwrap();
    for section in [
        "Goal:",
        "Inputs:",
        "Steps:",
        "Expected output:",
        "Done when:",
    ] {
        assert!(prompt.contains(section), "planning prompt misses {section}");
    }
    assert!(prompt.contains("same language as the goal"));
    assert!(prompt.contains("- "), "bullets requested");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn instruction_editor_starts_at_six_rows_and_grows_to_thirty() {
    assert!(SCRIPT.contains("key==='prompt'?6:2"));
    let fit = &SCRIPT[SCRIPT.find("function fitRows(").expect("auto-size helper")..];
    let fit = &fit[..fit.find('}').unwrap()];
    assert!(fit.contains("30") && fit.contains("6"), "{fit}");
}
