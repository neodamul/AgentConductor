use agentconductor::{
    adapters,
    plan::{AgentProfile, CatalogOrigin},
    protocol::{Decoder, EventKind, Provider},
};
use serde_json::{Value, json};

fn profile() -> AgentProfile {
    serde_json::from_value(json!({
        "id":"aipro", "executable":"aipro", "parser":"custom", "args":["run"],
        "catalog":{"models":["small","large"],"source":"operator","queried_at":0,"cli_version":"manual","origin":"manual"},
        "model_discovery":{"args":["models","--json"],"models_pointer":"/items","model_pointer":"/id"},
        "execution_options":[
            {"name":"model","args":["--model","{value}"],"default":"small"},
            {"name":"reasoning","args":["--reasoning","{value}"],"values":["low","high"],"default":"low","model_values":{"small":["low"],"large":["high"]}},
            {"name":"resume","args":["--resume","{session_id}"]}
        ],
        "result_pointer":"/data", "error_condition":{"pointer":"/status","equals":"error"}
    })).unwrap()
}

#[test]
fn custom_profile_keeps_catalog_discovery_options_and_output_rules_through_json_roundtrip() {
    let profile = profile();
    assert_eq!(
        profile.catalog.as_ref().unwrap().origin,
        CatalogOrigin::Manual
    );
    let saved = serde_json::to_value(&profile).unwrap();
    let reopened: AgentProfile = serde_json::from_value(saved).unwrap();
    assert_eq!(reopened.model_discovery.unwrap().models_pointer, "/items");
    assert_eq!(reopened.execution_options.len(), 3);
    assert_eq!(reopened.result_pointer.as_deref(), Some("/data"));
    assert_eq!(reopened.error_condition.unwrap().equals, json!("error"));
}

#[test]
fn generic_option_groups_resolve_defaults_overrides_and_model_restrictions() {
    let profile = profile();
    let args = adapters::session_args_with_options(
        &profile,
        Some("large"),
        Some("session-4"),
        "task.json",
        &[("reasoning".into(), "high".into())].into(),
    )
    .unwrap();
    assert_eq!(
        args,
        [
            "run",
            "--model",
            "large",
            "--reasoning",
            "high",
            "--resume",
            "session-4"
        ]
    );
    let error = adapters::session_args_with_options(
        &profile,
        Some("small"),
        None,
        "task.json",
        &[("reasoning".into(), "high".into())].into(),
    )
    .unwrap_err();
    assert!(error.contains("reasoning"));
}

#[test]
fn unset_option_groups_are_omitted_and_resume_uses_the_exact_session() {
    let mut profile = profile();
    profile.execution_options.push(
        serde_json::from_value(json!({
            "name":"format", "args":["--format", "{value}"], "values":["json", "text"]
        }))
        .unwrap(),
    );
    let args = adapters::session_args_with_options(
        &profile,
        Some("small"),
        None,
        "task.json",
        &Default::default(),
    )
    .unwrap();
    assert_eq!(args, ["run", "--model", "small", "--reasoning", "low"]);
    let resumed = adapters::session_args_with_options(
        &profile,
        Some("small"),
        Some("resume-42"),
        "task.json",
        &Default::default(),
    )
    .unwrap();
    assert!(
        resumed
            .windows(2)
            .any(|pair| pair == ["--resume", "resume-42"])
    );
    assert!(!resumed.iter().any(|arg| arg == "--format"));
}

#[test]
fn custom_result_pointer_and_error_condition_apply_before_event_mapping() {
    let profile = profile();
    let mut decoder = Decoder::for_profile(&profile, 1024);
    decoder
        .push(br#"{"type":"result","status":"error","data":{"message":"denied"}}"#)
        .unwrap();
    let events = decoder.finish().unwrap();
    let event = &events[0];
    assert_eq!(event.kind, EventKind::Error);
    assert_eq!(
        event.payload,
        json!({"type":"result","status":"error","data":{"message":"denied"}})
    );
    let success = Decoder::for_profile(&profile, 1024)
        .push(br#"{"type":"result","status":"success","data":{"answer":"ok"}}"#)
        .unwrap();
    assert!(success.is_empty());
    let mut success_decoder = Decoder::for_profile(&profile, 1024);
    success_decoder
        .push(br#"{"type":"result","status":"success","data":{"answer":"ok"}}"#)
        .unwrap();
    let success = success_decoder.finish().unwrap();
    assert_eq!(success[0].kind, EventKind::Result);
    assert_eq!(success[0].payload, json!({"answer":"ok"}));
}

#[test]
fn custom_result_error_without_data_preserves_the_error_event() {
    let profile = profile();
    let mut decoder = Decoder::for_profile(&profile, 1024);
    decoder
        .push(br#"{"type":"result","status":"error","message":"denied"}"#)
        .unwrap();
    let events = decoder.finish().unwrap();
    assert_eq!(events[0].kind, EventKind::Error);
    assert_eq!(events[0].payload["message"], "denied");
}

#[test]
fn result_pointer_is_used_by_the_planning_document_reader() {
    let profile = profile();
    let path = std::env::temp_dir().join(format!("ac-result-pointer-{}.jsonl", std::process::id()));
    std::fs::write(
        &path,
        br#"{"type":"result","status":"success","data":{"plan":"value"}}"#,
    )
    .unwrap();
    let document =
        agentconductor::coordinator::read_profile_document(&profile, &path, "plan").unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(document["plan"], "value");
}

#[test]
fn custom_catalog_discovery_uses_declared_command_and_response_mapping() {
    let profile = profile();
    let response: Value = json!({"items":[{"id":"large"},{"id":"small"}]});
    assert_eq!(
        agentconductor::models::parse_custom_catalog(&profile, &response).unwrap(),
        ["large", "small"]
    );
}

#[test]
fn built_in_profiles_do_not_gain_custom_cli_rules() {
    let mut profile = profile();
    profile.parser = Provider::Codex;
    assert!(profile.validate_custom_configuration().is_err());
}

#[test]
fn task_options_are_exportable_and_rejected_when_invalid_for_selected_model() {
    let profile = serde_json::to_value(profile()).unwrap();
    let mut plan: agentconductor::plan::Plan = serde_json::from_value(json!({
        "goal":"configure aipro", "agents":[profile], "tasks":[{
            "id":"one","agent":"aipro","prompt":"work","paths":["src/one"],
            "test_command":["cargo","test"],"model":"large",
            "execution_options":{"reasoning":"high"}
        }]
    }))
    .unwrap();
    plan.validate().unwrap();
    let restored: agentconductor::plan::Plan =
        serde_json::from_value(serde_json::to_value(&plan).unwrap()).unwrap();
    assert_eq!(restored.tasks[0].execution_options["reasoning"], "high");
    plan.tasks[0]
        .execution_options
        .insert("reasoning".into(), "low".into());
    assert!(plan.validate().unwrap_err().contains("invalid reasoning"));
}

#[test]
fn reassignment_revalidates_persisted_options_against_the_new_agent() {
    let profile = profile();
    let mut replacement = profile.clone();
    replacement.id = "replacement".into();
    replacement.catalog.as_mut().unwrap().models = vec!["small".into()];
    replacement
        .execution_options
        .iter_mut()
        .find(|option| option.name == "reasoning")
        .unwrap()
        .model_values = [("small".into(), vec!["low".into()])].into();
    let mut plan: agentconductor::plan::Plan = serde_json::from_value(json!({
        "goal":"configure aipro", "agents":[profile, replacement], "tasks":[{
            "id":"one","agent":"aipro","prompt":"work","paths":["src/one"],
            "test_command":["cargo","test"],"model":"large",
            "execution_options":{"reasoning":"high"}
        }]
    }))
    .unwrap();
    plan.validate().unwrap();
    plan.tasks[0].agent = "replacement".into();
    assert!(plan.validate().unwrap_err().contains("model"));
}
