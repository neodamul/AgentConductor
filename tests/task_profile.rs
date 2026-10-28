use agentconductor::{
    plan::{AgentProfile, Plan, Task, TaskType, WorkspaceMode},
    task_profile::{self, Warning},
};
use serde_json::json;

fn agent(model_values: serde_json::Value) -> AgentProfile {
    serde_json::from_value(json!({
        "id":"worker", "executable":"agent", "args":["run", "{task_file}", "{model}"], "parser":"custom",
        "execution_options":[{"name":"effort","args":["--effort","{value}"],"values":["low","medium","high"],"model_values":model_values}]
    })).unwrap()
}

fn task(task_type: Option<TaskType>, model: Option<&str>, options: serde_json::Value) -> Task {
    serde_json::from_value(json!({
        "id":"task", "agent":"worker", "prompt":"work", "paths":["src"], "test_command":["cargo","test"],
        "task_type":task_type.map(|value| serde_json::to_value(value).unwrap()),
        "model":model, "execution_options":options
    })).unwrap()
}

#[test]
fn effort_defaults_follow_task_type_and_operator_choices_win() {
    for (kind, expected) in [
        (TaskType::Extract, "low"),
        (TaskType::Summarize, "medium"),
        (TaskType::Analyze, "medium"),
        (TaskType::Code, "high"),
        (TaskType::Agentic, "high"),
        (TaskType::Creative, "medium"),
    ] {
        let resolved =
            task_profile::resolve(&agent(json!({})), &task(Some(kind), None, json!({}))).unwrap();
        assert_eq!(
            resolved.options.get("effort").map(String::as_str),
            Some(expected)
        );
    }
    let resolved = task_profile::resolve(
        &agent(json!({})),
        &task(Some(TaskType::Code), None, json!({"effort":"medium"})),
    )
    .unwrap();
    assert_eq!(
        resolved.options.get("effort").map(String::as_str),
        Some("medium")
    );
}

#[test]
fn model_specific_effort_is_bounded_and_unselected_model_stays_unset() {
    let profile = agent(json!({"small":["low"],"large":["low","medium","high"]}));
    let resolved = task_profile::resolve(
        &profile,
        &task(Some(TaskType::Code), Some("small"), json!({})),
    )
    .unwrap();
    assert_eq!(
        resolved.options.get("effort").map(String::as_str),
        Some("low")
    );
    assert_eq!(
        resolved.warnings,
        [Warning::EffortUnsupported {
            asked: "high".into(),
            applied: Some("low".into()),
        }]
    );

    let unresolved =
        task_profile::resolve(&profile, &task(Some(TaskType::Code), None, json!({}))).unwrap();
    assert!(!unresolved.options.contains_key("effort"));
}

#[test]
fn agent_task_defaults_choose_a_catalog_model_and_effort() {
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"worker", "executable":"agent", "args":["run", "{model}"], "parser":"custom",
        "catalog":{"models":["small"],"source":"cli","queried_at":1,"cli_version":"1"},
        "execution_options":[{"name":"effort","args":["--effort","{value}"],"values":["low","medium","high"],"model_values":{"small":["low","medium"]}}],
        "agent_settings":{"capabilities":{
            "task_defaults":{"extract":{"model":"small","effort":"medium"}},
            "models":[{"id":"small","traits":{"best_for":["extract"]}}]
        }}
    })).unwrap();
    let resolved =
        task_profile::resolve(&profile, &task(Some(TaskType::Extract), None, json!({}))).unwrap();
    assert_eq!(resolved.model.as_deref(), Some("small"));
    assert_eq!(
        resolved.options.get("effort").map(String::as_str),
        Some("medium")
    );
    assert!(resolved.warnings.is_empty());
}

#[test]
fn model_recommendation_reports_a_task_type_mismatch() {
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"worker", "executable":"agent", "args":["run", "{model}"], "parser":"custom",
        "catalog":{"models":["large"],"source":"cli","queried_at":1,"cli_version":"1"},
        "agent_settings":{"capabilities":{
            "task_defaults":{"code":{"model":"large"}},
            "models":[{"id":"large","traits":{"best_for":["analyze"]}}]
        }}
    }))
    .unwrap();
    let resolved =
        task_profile::resolve(&profile, &task(Some(TaskType::Code), None, json!({}))).unwrap();
    assert_eq!(
        resolved.warnings,
        [Warning::ModelNotSuited {
            model: "large".into(),
            task_type: TaskType::Code,
        }]
    );
}

#[test]
fn review_summary_exposes_effective_model_effort_and_warnings() {
    let profile: AgentProfile = serde_json::from_value(json!({
        "id":"worker", "executable":"agent", "args":["run", "{model}"], "parser":"custom",
        "catalog":{"models":["large"],"source":"cli","queried_at":1,"cli_version":"1"},
        "execution_options":[{"name":"effort","args":["--effort","{value}"],"values":["low","medium","high"]}],
        "agent_settings":{"capabilities":{
            "task_defaults":{"code":{"model":"large","effort":"high"}},
            "models":[{"id":"large","traits":{"best_for":["analyze"]}}]
        }}
    })).unwrap();
    let plan = Plan {
        goal: "goal".into(),
        plan_rules: String::new(),
        workspace_mode: WorkspaceMode::Shared,
        agents: vec![profile],
        tasks: vec![task(Some(TaskType::Code), None, json!({}))],
    };
    let summaries = task_profile::summaries(&plan.agents, &plan.tasks);
    assert_eq!(summaries[0]["task_id"], "task");
    assert_eq!(summaries[0]["model"], "large");
    assert_eq!(summaries[0]["options"]["effort"], "high");
    assert_eq!(summaries[0]["warnings"][0]["code"], "model_not_suited");
}

#[test]
fn resolved_effort_is_applied_to_the_cli_arguments() {
    let profile = agent(json!({}));
    let task = task(Some(TaskType::Extract), None, json!({}));
    let resolved = task_profile::resolve(&profile, &task).unwrap();
    let args = agentconductor::adapters::session_args_with_options(
        &profile,
        resolved.model.as_deref(),
        None,
        "prompt.md",
        &resolved.options,
    )
    .unwrap();
    assert!(
        args.windows(2).any(|pair| pair == ["--effort", "low"]),
        "{args:?}"
    );
}
