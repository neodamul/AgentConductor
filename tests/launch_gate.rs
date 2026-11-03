//! GitHub #149 / R-171 / DD-182: a run that cannot work is refused before anything is created.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{workflow, workflow::decisions, workflow::guardrail};

#[tokio::test]
async fn a_missing_test_tool_refuses_the_run_and_leaves_nothing_behind() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks[0].test_command = vec!["definitely-not-installed-tool".into(), "--check".into()];
    let run = root.join("run");
    let error = refused(workflow::run(plan, &root.join("repo"), &run, 10).await);
    assert!(error.contains("launch gate"), "{error}");
    assert!(error.contains("[test_unrunnable]"), "{error}");
    assert!(error.contains("definitely-not-installed-tool"), "{error}");
    assert!(error.contains(&plan_task_id(0)), "{error}");
    assert!(!run.exists(), "no run folder is created");
    std::fs::remove_dir_all(root).unwrap();
}

fn refused(result: Result<workflow::Report, String>) -> String {
    match result {
        Err(error) => error,
        Ok(_) => panic!("the run should have been refused"),
    }
}

fn plan_task_id(index: usize) -> String {
    ["one.txt", "two.txt", "three.txt"][index].into()
}

#[tokio::test]
async fn every_violation_is_listed_at_once() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].executable = "no-such-agent-binary".into();
    plan.tasks[1].test_command = vec!["another-missing-tool".into()];
    let error = refused(workflow::run(plan, &root.join("repo"), &root.join("run"), 10).await);
    assert!(
        error.contains("[tool_missing]") && error.contains("no-such-agent-binary"),
        "{error}"
    );
    assert!(
        error.contains("[test_unrunnable]") && error.contains("another-missing-tool"),
        "{error}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_passed_gate_is_recorded_and_listed() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    let rows = decisions::list(&run).unwrap();
    let gate = rows.iter().find(|r| r["kind"] == "gate").expect("gate row");
    assert_eq!(gate["judgment"], "passed");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn executables_resolve_on_path_absolute_or_in_the_project() {
    let root = std::env::temp_dir().join(format!(
        "ac-gate-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(root.join("scripts")).unwrap();
    std::fs::write(root.join("scripts/check.sh"), "#!/bin/sh\n").unwrap();
    assert!(guardrail::resolvable("sh", &root));
    assert!(guardrail::resolvable("/bin/sh", &root));
    assert!(guardrail::resolvable("scripts/check.sh", &root));
    assert!(!guardrail::resolvable("scripts/missing.sh", &root));
    assert!(!guardrail::resolvable("", &root));
    assert!(!guardrail::resolvable(
        "definitely-not-installed-tool",
        &root
    ));
    std::fs::remove_dir_all(root).unwrap();
}

/// A script that a task is asked to write inside its owned paths is not missing yet.
#[tokio::test]
async fn a_test_script_written_by_a_task_does_not_block_the_run() {
    let (root, mut plan) = support::setup("shared");
    let repo = root.join("repo");
    plan.tasks[0].paths = vec!["scripts".into()];
    plan.tasks[0].test_command = vec!["./scripts/run_tests.sh".into()];
    let (checks, violations) = guardrail::launch_gate(&plan, &repo);
    assert!(checks >= 3 && violations.is_empty(), "{violations:?}");
    // A relative program outside every owned path is still missing.
    plan.tasks[0].test_command = vec!["./tools/run_tests.sh".into()];
    let (_, violations) = guardrail::launch_gate(&plan, &repo);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code, "test_unrunnable");
    std::fs::remove_dir_all(root).unwrap();
}
