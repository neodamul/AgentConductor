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
                model_discovery: None,
                execution_options: vec![],
                result_pointer: None,
                error_condition: None,
                max_sessions: 3,
                quota_refresh_minutes: 5,
                quota_group: None,
                fallback: None,
                cooperative: Default::default(),
                event_types: Default::default(),
                session_id_pointer: None,
                quota_command: None,
                protocol: None,
                agent_settings: None,
            })
            .collect(),
        tasks: ["src/auth", "tests/auth", "docs/auth"]
            .into_iter()
            .enumerate()
            .map(|(i, path)| Task {
                model: None,
                role: None,
                execution_options: Default::default(),
                task_type: None,
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
fn requires_one_to_twelve_tasks() {
    // R-53 supersedes the original fixed three tasks.
    let mut p = plan();
    p.tasks.pop();
    assert!(p.validate().is_ok(), "two tasks are a valid plan");
    p.tasks.clear();
    assert!(p.validate().is_err(), "an empty plan is refused");
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

#[test]
fn validates_custom_event_and_quota_settings_without_requiring_them_on_old_profiles() {
    let mut p = plan();
    assert!(p.validate().is_ok());
    p.agents[0].quota_command = Some(agentconductor::plan::QuotaCommand {
        args: vec!["quota".into()],
        timeout_seconds: 301,
        output_pointer: None,
    });
    assert!(p.validate().is_err());
    p.agents[0].quota_command.as_mut().unwrap().timeout_seconds = 30;
    p.agents[0].session_id_pointer = Some("not/a/pointer".into());
    assert!(p.validate().is_err());
}

/// GitHub #251 (DD-202): a rejected task is named together with the field that is wrong.
#[test]
fn a_rejected_task_is_named_with_its_field() {
    type Case = (&'static str, fn(&mut Task), &'static str);
    let cases: [Case; 5] = [
        (
            "empty paths",
            |t| t.paths.clear(),
            "task task-1: paths is empty",
        ),
        (
            "empty prompt",
            |t| t.prompt = " ".into(),
            "task task-1: prompt is empty",
        ),
        (
            "unknown agent",
            |t| t.agent = "nobody".into(),
            "task task-1: unknown agent nobody",
        ),
        (
            "no test command",
            |t| t.test_command.clear(),
            "task task-1: test_command",
        ),
        (
            "duplicate id",
            |t| t.id = "task-0".into(),
            "task task-0: duplicate task id",
        ),
    ];
    for (label, change, expected) in cases {
        let mut plan = plan();
        change(&mut plan.tasks[1]);
        let error = plan.validate().unwrap_err();
        assert!(error.contains(expected), "{label}: {error}");
    }
    let mut plan = plan();
    plan.tasks[1].id = " ".into();
    assert!(plan.validate().unwrap_err().contains("task id is empty"));
}
