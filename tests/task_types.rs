//! GitHub #210: a plan task says what kind of work it is, and the effort and model it runs with are resolved from
//! that type and the agent's settings instead of being the same for every task and model.
use agentconductor::{
    adapters,
    plan::{AgentProfile, Task, TaskType},
    planner::schema,
    task_profile::{self, Warning},
};
use serde_json::json;

fn agent(extra: serde_json::Value) -> AgentProfile {
    let mut value = json!({
        "id":"a","executable":"/bin/agent","args":["run","{task_file}","{model}"],"parser":"custom",
        "catalog":{"source":"fixture","models":["big","small"],"queried_at":0,"cli_version":"1"},
        "execution_options":[{"name":"effort","args":["--effort","{value}"],"values":["low","medium","high"],"model_values":{"big":["low","medium","high"],"small":["low"]}}]
    });
    value
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(value).unwrap()
}
fn task(value: serde_json::Value) -> Task {
    let mut base =
        json!({"id":"t1","agent":"a","prompt":"p","paths":["a.txt"],"test_command":["true"]});
    base.as_object_mut()
        .unwrap()
        .extend(value.as_object().unwrap().clone());
    serde_json::from_value(base).unwrap()
}
fn effort(r: &task_profile::Resolved) -> Option<&str> {
    r.options.get("effort").map(String::as_str)
}

#[test]
fn a_task_type_is_optional_and_does_not_change_older_digests() {
    let plain = task(json!({}));
    assert_eq!(plain.task_type, None);
    assert!(
        serde_json::to_value(&plain)
            .unwrap()
            .get("task_type")
            .is_none(),
        "absent stays absent"
    );
    let typed = task(json!({"task_type":"agentic"}));
    assert_eq!(typed.task_type, Some(TaskType::Agentic));
    assert_eq!(
        serde_json::to_value(&typed).unwrap()["task_type"],
        "agentic"
    );
    assert!(serde_json::from_value::<Task>(json!({"id":"t","agent":"a","prompt":"p","paths":[],"test_command":[],"task_type":"poetry"})).is_err());
}

#[test]
fn the_coordinator_may_set_the_type_but_does_not_have_to() {
    let contract = schema::for_plan(&["a".into()], &[], false);
    let task = &contract["properties"]["tasks"]["items"];
    assert_eq!(
        task["properties"]["task_type"]["enum"],
        json!([
            "extract",
            "summarize",
            "analyze",
            "code",
            "agentic",
            "creative",
            null
        ])
    );
    assert!(
        !task["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|k| k == "task_type")
    );
}

#[test]
fn the_effort_follows_the_task_type_when_the_operator_chose_nothing() {
    for (kind, expected) in [
        ("extract", "low"),
        ("summarize", "medium"),
        ("analyze", "medium"),
        ("code", "high"),
        ("agentic", "high"),
        ("creative", "medium"),
    ] {
        let r = task_profile::resolve(
            &agent(json!({})),
            &task(json!({"task_type":kind,"model":"big"})),
        )
        .unwrap();
        assert_eq!(effort(&r), Some(expected), "{kind}");
        assert!(r.warnings.is_empty(), "{kind}: {:?}", r.warnings);
    }
    let untyped = task_profile::resolve(&agent(json!({})), &task(json!({"model":"big"}))).unwrap();
    assert_eq!(
        untyped.task_type,
        TaskType::Code,
        "a task without a type is code"
    );
    assert_eq!(effort(&untyped), Some("high"));
}

#[test]
fn the_operators_choice_wins_and_other_options_pass_through() {
    let t = task(
        json!({"task_type":"code","model":"big","execution_options":{"effort":"low","other":"x"}}),
    );
    let r = task_profile::resolve(&agent(json!({})), &t).unwrap();
    assert_eq!(effort(&r), Some("low"));
    assert_eq!(r.options["other"], "x");
    assert!(r.warnings.is_empty());
}

#[test]
fn an_effort_the_model_does_not_honor_is_lowered_with_a_warning() {
    let r = task_profile::resolve(
        &agent(json!({})),
        &task(json!({"task_type":"code","model":"small"})),
    )
    .unwrap();
    assert_eq!(effort(&r), Some("low"));
    assert_eq!(
        r.warnings,
        [Warning::EffortUnsupported {
            asked: "high".into(),
            applied: Some("low".into())
        }]
    );
    let none = agent(
        json!({"execution_options":[{"name":"effort","args":["--effort","{value}"],"values":["low"],"model_values":{"big":[]}}]}),
    );
    let r = task_profile::resolve(&none, &task(json!({"task_type":"code","model":"big"}))).unwrap();
    assert_eq!(effort(&r), None);
    assert_eq!(
        r.warnings,
        [Warning::EffortUnsupported {
            asked: "high".into(),
            applied: None
        }]
    );
}

#[test]
fn an_agent_without_an_effort_setting_gets_none_and_no_warning() {
    let bare = agent(json!({"execution_options":[]}));
    let r = task_profile::resolve(&bare, &task(json!({"task_type":"code","model":"big"}))).unwrap();
    assert!(r.options.is_empty() && r.warnings.is_empty());
}

#[test]
fn the_agents_own_task_defaults_replace_the_built_in_table() {
    let settings = json!({"schema":"acio.agent/1","capabilities":{"read_only":true,"streaming":false,"task_defaults":{"code":{"effort":"medium","model":"small"}}},"invocation":{"input":"stdin"}});
    let a = agent(json!({"agent_settings":settings}));
    let r = task_profile::resolve(&a, &task(json!({"task_type":"code"}))).unwrap();
    assert_eq!(
        r.model.as_deref(),
        Some("small"),
        "the agent's recommended model when the task names none"
    );
    assert_eq!(
        effort(&r),
        Some("low"),
        "medium is not honored by small, so it is lowered"
    );
    let named =
        task_profile::resolve(&a, &task(json!({"task_type":"code","model":"big"}))).unwrap();
    assert_eq!(named.model.as_deref(), Some("big"));
    assert_eq!(effort(&named), Some("medium"));
    let unknown = agent(
        json!({"agent_settings":{"schema":"acio.agent/1","capabilities":{"read_only":true,"streaming":false,"task_defaults":{"code":{"model":"ghost"}}},"invocation":{"input":"stdin"}}}),
    );
    assert_eq!(
        task_profile::resolve(&unknown, &task(json!({"task_type":"code"})))
            .unwrap()
            .model,
        None,
        "a model outside the catalog is never chosen"
    );
}

#[test]
fn a_model_that_does_not_suit_the_task_type_is_reported_not_blocked() {
    let settings = json!({"schema":"acio.agent/1","capabilities":{"read_only":true,"streaming":false,"models":[{"id":"small","traits":{"reasoning":false,"efforts":[],"best_for":["extract"]}}]},"invocation":{"input":"stdin"}});
    let a = agent(json!({"agent_settings":settings}));
    let r =
        task_profile::resolve(&a, &task(json!({"task_type":"agentic","model":"small"}))).unwrap();
    assert!(r.warnings.contains(&Warning::ModelNotSuited {
        model: "small".into(),
        task_type: TaskType::Agentic
    }));
    let fine =
        task_profile::resolve(&a, &task(json!({"task_type":"extract","model":"small"}))).unwrap();
    assert!(
        !fine
            .warnings
            .iter()
            .any(|w| matches!(w, Warning::ModelNotSuited { .. }))
    );
}

#[test]
fn the_resolved_options_become_arguments() {
    let a = agent(json!({}));
    let r = task_profile::resolve(&a, &task(json!({"task_type":"extract","model":"big"}))).unwrap();
    let args =
        adapters::session_args_with_options(&a, r.model.as_deref(), None, "prompt.md", &r.options)
            .unwrap();
    assert!(
        args.windows(2).any(|p| p == ["--effort", "low"]),
        "{args:?}"
    );
}
