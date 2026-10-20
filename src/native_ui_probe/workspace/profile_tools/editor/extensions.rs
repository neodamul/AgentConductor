use serde_json::Value;

pub(super) fn serialize(profile: &Value) -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "catalog":profile["catalog"], "model_discovery":profile["model_discovery"],
        "execution_options":profile["execution_options"], "result_pointer":profile["result_pointer"],
        "error_condition":profile["error_condition"]
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
        "catalog",
        "model_discovery",
        "execution_options",
        "result_pointer",
        "error_condition",
    ] {
        if let Some(value) = values.get(key) {
            profile[key] = if key == "execution_options" && value.is_null() {
                serde_json::json!([])
            } else {
                value.clone()
            };
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
