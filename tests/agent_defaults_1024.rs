//! GitHub #1024: an agent's chosen default model and effort apply to its tasks unless a task sets its own; an effort
//! the model does not honor is lowered with a warning; a default model the catalog no longer lists falls back.
use agentconductor::{
    plan::{AgentProfile, Task},
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
    let mut base = json!({"id":"t1","agent":"a","prompt":"p","paths":["a.txt"],"test_command":["true"],"task_type":"extract"});
    base.as_object_mut()
        .unwrap()
        .extend(value.as_object().unwrap().clone());
    serde_json::from_value(base).unwrap()
}

#[test]
fn the_agent_default_model_and_effort_apply_unless_the_task_sets_its_own() {
    let profile = agent(json!({"default_model":"big","default_effort":"high"}));
    let r = task_profile::resolve(&profile, &task(json!({}))).unwrap();
    assert_eq!(r.model.as_deref(), Some("big"));
    assert_eq!(
        r.options.get("effort").map(String::as_str),
        Some("high"),
        "not the extract default (low)"
    );
    let own = task_profile::resolve(
        &profile,
        &task(json!({"model":"small","execution_options":{"effort":"low"}})),
    )
    .unwrap();
    assert_eq!(own.model.as_deref(), Some("small"));
    assert_eq!(own.options.get("effort").map(String::as_str), Some("low"));
}

#[test]
fn an_unsupported_default_effort_is_lowered_with_a_warning() {
    let profile = agent(json!({"default_model":"small","default_effort":"high"}));
    let r = task_profile::resolve(&profile, &task(json!({}))).unwrap();
    assert_eq!(r.options.get("effort").map(String::as_str), Some("low"));
    assert!(
        r.warnings
            .iter()
            .any(|w| matches!(w, Warning::EffortUnsupported { .. }))
    );
}

#[test]
fn a_default_model_missing_from_the_catalog_falls_back_and_bad_efforts_are_refused() {
    let gone = agent(json!({"default_model":"retired"}));
    assert_eq!(
        task_profile::resolve(&gone, &task(json!({})))
            .unwrap()
            .model,
        None
    );
    assert!(
        agent(json!({"default_effort":"turbo"}))
            .validate_defaults()
            .is_err()
    );
    assert!(
        agent(json!({"default_effort":"max"}))
            .validate_defaults()
            .is_ok()
    );
    let old: AgentProfile = agent(json!({}));
    let text = serde_json::to_string(&old).unwrap();
    assert!(
        !text.contains("default_model") && !text.contains("default_effort"),
        "older profiles keep their shape"
    );
}
