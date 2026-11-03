use agentconductor::{coordinator, plan::AgentProfile};
use serde_json::{Value, json};

fn custom_profile(read_only_args: Value, coordinator_model: Value) -> AgentProfile {
    serde_json::from_value(json!({
        "id": "aipro",
        "executable": "aipro",
        "args": ["run", "--model", "{model}", "--allow-write", "{task_file}"],
        "parser": "custom",
        "permission_mode": "operator_auto_approve",
        "catalog": {
            "models": ["local/one"],
            "source": "aipro models",
            "queried_at": 1,
            "cli_version": "test"
        },
        "read_only_args": read_only_args,
        "coordinator_model": coordinator_model
    }))
    .unwrap()
}

#[test]
fn declared_read_only_argv_replaces_normal_custom_argv() {
    let profile = custom_profile(
        json!(["chat", "--read-only", "{model}", "{task_file}"]),
        json!("local/one"),
    );
    profile.validate_custom_configuration().unwrap();

    let args = coordinator::read_only_args(&profile, "prompt.json").unwrap();
    assert_eq!(args, ["chat", "--read-only", "local/one", "prompt.json"]);
    assert!(!args.iter().any(|arg| arg == "--allow-write"));
}

#[test]
fn unsafe_or_empty_custom_read_only_argv_is_rejected() {
    for read_only_args in [
        json!([]),
        json!(["run", "--model", "{model}", "--allow-write", "{task_file}"]),
    ] {
        let profile = custom_profile(read_only_args, Value::Null);
        assert!(profile.validate_custom_configuration().is_err());
        assert!(coordinator::read_only_args(&profile, "prompt.json").is_err());
    }
}

#[test]
fn selected_coordinator_model_must_belong_to_the_observed_catalog() {
    let profile = custom_profile(
        json!(["chat", "--read-only", "{model}", "{task_file}"]),
        json!("local/one"),
    );
    profile.validate_custom_configuration().unwrap();

    let invalid = custom_profile(
        json!(["chat", "--read-only", "{model}", "{task_file}"]),
        json!("local/unknown"),
    );
    assert!(invalid.validate_custom_configuration().is_err());
}
