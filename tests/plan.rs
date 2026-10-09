use agentconductor::plan::{AgentProfile, Plan, Task};

fn plan() -> Plan {
    Plan {
        goal: "refresh token rotation".into(),
        agents: ["claude", "codex", "opencode"]
            .into_iter()
            .map(|id| AgentProfile {
                id: id.into(),
                executable: id.into(),
                args: vec![],
            })
            .collect(),
        tasks: ["src/auth", "tests/auth", "docs/auth"]
            .into_iter()
            .enumerate()
            .map(|(i, path)| Task {
                id: format!("task-{i}"),
                agent: ["claude", "codex", "opencode"][i].into(),
                prompt: "Implement the assigned scope".into(),
                paths: vec![path.into()],
                depends_on: vec![],
                test_command: vec!["cargo".into(), "test".into()],
            })
            .collect(),
    }
}

#[test]
fn accepts_three_independent_scopes() {
    assert!(plan().validate().is_ok());
}

#[test]
fn requires_exactly_three_tasks() {
    let mut p = plan();
    p.tasks.pop();
    assert!(p.validate().is_err());
}

#[test]
fn rejects_unknown_agent_and_duplicate_ids() {
    let mut p = plan();
    p.tasks[0].agent = "missing".into();
    assert!(p.validate().is_err());
    let mut p = plan();
    p.tasks[1].id = p.tasks[0].id.clone();
    assert!(p.validate().is_err());
    let mut p = plan();
    p.agents[1].id = p.agents[0].id.clone();
    assert!(p.validate().is_err());
}

#[test]
fn rejects_cycles_and_unknown_dependencies() {
    let mut p = plan();
    p.tasks[0].depends_on = vec!["task-1".into()];
    p.tasks[1].depends_on = vec!["task-0".into()];
    assert!(p.validate().is_err());
    let mut p = plan();
    p.tasks[0].depends_on = vec!["missing".into()];
    assert!(p.validate().is_err());
}

#[test]
fn rejects_overlapping_scopes_and_path_escape() {
    for bad in [
        "src/auth",
        "src/auth/token.rs",
        "../secret",
        "/etc",
        "src/../auth",
        "src\\auth",
        "src/*",
        ".git/config",
    ] {
        let mut p = plan();
        p.tasks[1].paths = vec![bad.into()];
        assert!(p.validate().is_err(), "accepted {bad}");
    }
}

#[test]
fn scope_comparison_respects_path_components() {
    let mut p = plan();
    p.tasks[1].paths = vec!["src/authentication".into()];
    assert!(p.validate().is_ok());
}

#[test]
fn rejects_empty_inputs_and_shell_executable() {
    let mut p = plan();
    p.goal.clear();
    assert!(p.validate().is_err());
    let mut p = plan();
    p.tasks[0].paths.clear();
    assert!(p.validate().is_err());
    let mut p = plan();
    p.agents[0].executable = "sh".into();
    assert!(p.validate().is_err());
}

#[test]
fn accepts_a_self_hosted_agent_without_vendor_specific_fields() {
    let mut p = plan();
    p.agents[2].id = "aipro".into();
    p.agents[2].executable = "aipro".into();
    p.tasks[2].agent = "aipro".into();
    assert!(p.validate().is_ok());
}
