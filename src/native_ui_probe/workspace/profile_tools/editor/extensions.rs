use serde_json::Value;

pub(super) fn configuration_hint(profile: &Value, input: &str) -> Option<&'static str> {
    let rules: Value = serde_json::from_str(input).ok()?;
    let mut profile = profile.as_object()?.clone();
    for key in ["event_types", "error_condition", "denial_condition"] {
        if let Some(value) = rules.get(key) {
            profile.insert(key.to_owned(), value.clone());
        }
    }
    serde_json::from_value::<crate::plan::AgentProfile>(Value::Object(profile))
        .ok()?
        .configuration_hint()
}

pub(super) fn serialize(profile: &Value) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "agent_settings":profile["agent_settings"], "catalog":profile["catalog"],
        "model_discovery":profile["model_discovery"],
        "execution_options":profile["execution_options"], "result_pointer":profile["result_pointer"],
        "error_condition":profile["error_condition"], "on_invalid_json":profile["on_invalid_json"],
        "denial_condition":profile["denial_condition"], "quota_command":profile["quota_command"],
        "quota_rules":profile["quota_rules"].as_array().cloned().unwrap_or_default()
    }))
    .unwrap_or_else(|_| "{}".into())
}

pub(super) fn apply(
    profile: &mut Value,
    input: &str,
    original: &Value,
    provider_changed: bool,
) -> Result<(), String> {
    let values: Value =
        serde_json::from_str(input).map_err(|error| format!("invalid profile rules: {error}"))?;
    let Some(values) = values.as_object() else {
        return Err("profile rules must be a JSON object".into());
    };
    for key in [
        "agent_settings",
        "catalog",
        "model_discovery",
        "execution_options",
        "result_pointer",
        "error_condition",
        "on_invalid_json",
        "denial_condition",
        "quota_command",
        "quota_rules",
    ] {
        if let Some(value) = values.get(key) {
            if key == "on_invalid_json" && value.is_null() {
                if let Some(object) = profile.as_object_mut() {
                    object.remove(key);
                }
            } else {
                profile[key] =
                    if matches!(key, "execution_options" | "quota_rules") && value.is_null() {
                        serde_json::json!([])
                    } else {
                        value.clone()
                    };
            }
        }
    }
    if provider_changed {
        profile["catalog"] = Value::Null;
    } else if profile["catalog"]["models"] != original["catalog"]["models"]
        && let Some(catalog) = profile["catalog"].as_object_mut()
    {
        catalog.insert("origin".into(), "manual".into());
        catalog.insert("source".into(), "operator-configured catalog".into());
        catalog.insert("queried_at".into(), 0.into());
        catalog.insert("cli_version".into(), "manual".into());
    }
    let parsed: crate::plan::AgentProfile = serde_json::from_value(profile.clone())
        .map_err(|error| format!("invalid profile configuration: {error}"))?;
    parsed.validate_custom_configuration()?;
    if let Some(model) = parsed
        .execution_options
        .iter()
        .find(|option| option.name == "model")
        .and_then(|option| option.default.as_deref())
    {
        parsed.validate_model(model)?;
    }
    Ok(())
}
