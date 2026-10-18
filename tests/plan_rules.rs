#![cfg(unix)]
//! R-59 / DD-65 / QAS-50: editable plan rules reach the planning coordinator.
use agentconductor::plan_rules;
use std::{process::Command, sync::Mutex};
#[path = "support/workflow.rs"]
mod support;
static HOME_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn global_rules_are_created_once_and_workspace_text_overrides_them() {
    let _home = HOME_LOCK.lock().unwrap();
    let (root, _) = support::setup("shared");
    let home = root.join("home");
    // The lock keeps global-path lookups in this binary on the same fixture HOME.
    unsafe { std::env::set_var("HOME", &home) };
    let path = plan_rules::global_path().unwrap();
    assert_eq!(path, home.join(".agentconductor/plan-rules.md"));
    let rules = plan_rules::effective(None).unwrap();
    for topic in ["analysis", "phase", "verification", "500", "English", "TDD"] {
        assert!(rules.contains(topic), "default rules mention {topic}");
    }
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

#[test]
fn repository_rules_are_loaded_recursively_in_relative_path_order() {
    let _home = HOME_LOCK.lock().unwrap();
    let (root, _) = support::setup("shared");
    let repository = root.join("repo");
    let rules = repository.join(".agentconductor/plan-rules");
    std::fs::create_dir_all(&rules).unwrap();
    assert_eq!(
        plan_rules::for_repository(None, &repository).unwrap(),
        plan_rules::current(None),
        "an empty rules directory adds no content"
    );
    std::fs::create_dir_all(rules.join("nested")).unwrap();
    std::fs::write(rules.join("z.md"), "Z rule").unwrap();
    std::fs::write(rules.join("nested/b.md"), "B rule").unwrap();
    std::fs::write(rules.join("a.md"), "A rule").unwrap();
    std::fs::write(rules.join("ignored.txt"), "not a rule").unwrap();

    let effective = plan_rules::for_repository(None, &repository).unwrap();
    let a = effective.find("A rule").unwrap();
    let b = effective.find("B rule").unwrap();
    let z = effective.find("Z rule").unwrap();
    assert!(a < b && b < z, "rules are ordered by relative path: {effective}");
    assert!(!effective.contains("not a rule"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn repository_rules_keep_existing_global_and_inline_precedence() {
    let _home = HOME_LOCK.lock().unwrap();
    let (root, _) = support::setup("shared");
    let repository = root.join("repo");
    let rules = repository.join(".agentconductor/plan-rules");
    std::fs::create_dir_all(&rules).unwrap();
    std::fs::write(rules.join("repo.md"), "Repository rule").unwrap();
    let home = root.join("home");
    unsafe { std::env::set_var("HOME", &home) };
    std::fs::create_dir_all(home.join(".agentconductor")).unwrap();
    std::fs::write(home.join(".agentconductor/plan-rules.md"), "Global rule").unwrap();

    let combined = plan_rules::for_repository(None, &repository).unwrap();
    assert!(combined.contains("Global rule"));
    assert!(combined.contains("Repository rule"));
    assert_eq!(
        plan_rules::for_repository(Some("Inline only"), &repository).unwrap(),
        "Inline only"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn repository_rules_ignore_files_outside_root_and_symlinks() {
    let _home = HOME_LOCK.lock().unwrap();
    use std::os::unix::fs::symlink;

    let (root, _) = support::setup("shared");
    let repository = root.join("repo");
    let rules = repository.join(".agentconductor/plan-rules");
    std::fs::create_dir_all(&rules).unwrap();
    let outside = root.join("outside.md");
    std::fs::write(&outside, "Outside rule").unwrap();
    symlink(&outside, rules.join("linked.md")).unwrap();
    let outside_dir = root.join("outside");
    std::fs::create_dir_all(&outside_dir).unwrap();
    std::fs::write(outside_dir.join("nested.md"), "External nested rule").unwrap();
    symlink(&outside_dir, rules.join("linked-dir")).unwrap();

    let linked_repository = root.join("linked-repo");
    std::fs::create_dir_all(&linked_repository).unwrap();
    let linked_metadata = root.join("external-metadata");
    std::fs::create_dir_all(linked_metadata.join("plan-rules")).unwrap();
    std::fs::write(
        linked_metadata.join("plan-rules/hidden.md"),
        "Linked metadata rule",
    )
    .unwrap();
    symlink(&linked_metadata, linked_repository.join(".agentconductor")).unwrap();

    let effective = plan_rules::for_repository(None, &repository).unwrap();
    let linked = plan_rules::for_repository(None, &linked_repository).unwrap();
    assert!(!effective.contains("Outside rule"));
    assert!(!effective.contains("External nested rule"));
    assert!(!linked.contains("Linked metadata rule"));
    std::fs::remove_dir_all(root).unwrap();
}
