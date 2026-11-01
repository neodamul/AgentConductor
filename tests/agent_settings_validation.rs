use agentconductor::plan::AgentProfile;

#[test]
fn profile_rejects_agent_settings_that_violate_the_agent_schema() {
    let profile: AgentProfile = serde_json::from_value(serde_json::json!({
        "id":"claude",
        "executable":"claude",
        "args":["-p"],
        "parser":"claude",
        "agent_settings":{"invocation":{"session":{"resume":{"mode":"replace_argv"}}}}
    }))
    .unwrap();

    let error = profile.validate_custom_configuration().unwrap_err();
    assert!(error.contains("invalid agent settings"), "{error}");
    assert!(error.contains("argv"), "{error}");
}
