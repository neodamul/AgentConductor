use agentconductor::plan::{AgentProfile, Plan, Task};

fn plan() -> Plan {
    Plan {
        goal: "refresh token rotation".into(),
        workspace_mode: Default::default(),
        agents: ["claude", "codex", "opencode"]
            .into_iter()
            .map(|id| AgentProfile {
                id: id.into(),
                executable: id.into(),
                args: vec![],
                parser: Default::default(),
                input_mode: Default::default(),
                environment: Default::default(),
                permission_mode: "unverified".into(),
                catalog: None,
                max_sessions: 3,
                quota_group: None,
                fallback: None,
            })
            .collect(),
        tasks: ["src/auth", "tests/auth", "docs/auth"]
            .into_iter()
            .enumerate()
            .map(|(i, path)| Task {
                model: None,
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

#[test]
fn workspace_mode_defaults_to_shared_for_existing_plans() {
    let mut json = serde_json::to_value(plan()).unwrap();
    json.as_object_mut().unwrap().remove("workspace_mode");
    let decoded: Plan = serde_json::from_value(json).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap()["workspace_mode"],
        "shared"
    );
}

#[test]
fn worktree_mode_is_explicit_and_unknown_modes_are_rejected() {
    let mut json = serde_json::to_value(plan()).unwrap();
    json["workspace_mode"] = "worktree".into();
    let decoded: Plan = serde_json::from_value(json.clone()).unwrap();
    assert!(decoded.validate().is_ok());
    assert_eq!(
        serde_json::to_value(decoded).unwrap()["workspace_mode"],
        "worktree"
    );
    json["workspace_mode"] = "unsupported".into();
    assert!(serde_json::from_value::<Plan>(json).is_err());
}
