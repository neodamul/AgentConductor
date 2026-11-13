use super::*;

pub fn parse_catalog(provider: Provider, response: &Value) -> Result<Vec<String>, String> {
    let entries = match provider {
        Provider::Claude => response.pointer("/response/response/models"),
        Provider::Codex => response.pointer("/result/data"),
        _ => return Err("catalog JSON parsing is unsupported for this provider".into()),
    }
    .and_then(Value::as_array)
    .ok_or("CLI did not return a supported model catalog; import an operator-observed catalog")?;
    let mut models = vec![];
    for entry in entries {
        if entry.get("hidden").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let model = entry
            .get(if provider == Provider::Claude {
                "value"
            } else {
                "model"
            })
            .and_then(Value::as_str)
            .ok_or("catalog model ID missing")?;
        if model.is_empty() || model.starts_with('-') || model.chars().any(char::is_whitespace) {
            return Err("invalid catalog model ID".into());
        }
        if !models.iter().any(|m| m == model) {
            models.push(model.to_string());
        }
    }
    if models.is_empty() {
        return Err("CLI returned an empty model catalog".into());
    }
    Ok(models)
}

pub fn parse_custom_catalog(
    profile: &AgentProfile,
    response: &Value,
) -> Result<Vec<String>, String> {
    let config = profile
        .model_discovery
        .as_ref()
        .ok_or("custom catalog discovery is not configured")?;
    let entries = response
        .pointer(&config.models_pointer)
        .and_then(Value::as_array)
        .ok_or("configured model-list JSON Pointer did not select an array")?;
    let mut models = Vec::new();
    for entry in entries {
        let model = if config.model_pointer.is_empty() {
            entry.as_str()
        } else {
            entry.pointer(&config.model_pointer).and_then(Value::as_str)
        }
        .ok_or("configured model-id JSON Pointer did not select a string")?;
        if model.is_empty() || model.starts_with('-') || model.chars().any(char::is_whitespace) {
            return Err("invalid catalog model ID".into());
        }
        if !models.iter().any(|known| known == model) {
            models.push(model.to_owned());
        }
    }
    if models.is_empty() {
        return Err("CLI returned an empty model catalog".into());
    }
    Ok(models)
}
