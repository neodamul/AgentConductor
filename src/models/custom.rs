use crate::plan::{AgentProfile, CatalogOrigin, ModelCatalog};
use serde_json::Value;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn parse_custom_catalog(
    profile: &AgentProfile,
    response: &Value,
) -> Result<Vec<String>, String> {
    let discovery = profile
        .model_discovery
        .as_ref()
        .ok_or("custom model discovery is not configured")?;
    let entries = response
        .pointer(&discovery.models_pointer)
        .and_then(Value::as_array)
        .ok_or("custom model catalog array was not found")?;
    let mut models = Vec::new();
    for entry in entries {
        let model = if discovery.model_pointer.is_empty() {
            entry.as_str()
        } else {
            entry
                .pointer(&discovery.model_pointer)
                .and_then(Value::as_str)
        }
        .ok_or("custom catalog model ID missing")?;
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

pub async fn discover(profile: &AgentProfile, dir: &Path) -> Result<ModelCatalog, String> {
    use crate::process::{self, ProcessSpec};
    let discovery = profile
        .model_discovery
        .as_ref()
        .ok_or("custom agents need an observed catalog or model discovery command")?;
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    let args = discovery
        .args
        .iter()
        .map(|arg| arg.replace("{executable}", &profile.executable))
        .collect();
    let result = process::run(ProcessSpec {
        executable: profile.executable.clone().into(),
        args,
        cwd: dir.clone(),
        input: vec![],
        log_dir: dir.join("query"),
        timeout: std::time::Duration::from_secs(discovery.timeout_seconds.into()),
        signal_grace: std::time::Duration::from_secs(1),
        environment: profile.environment.clone(),
    })
    .await?;
    if result.exit_code != Some(0) || result.timed_out {
        return Err("custom model query failed; inspect discovery logs".into());
    }
    let bytes = std::fs::read(result.stdout).map_err(|e| e.to_string())?;
    let response: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("custom model query returned invalid JSON: {e}"))?;
    let models = parse_custom_catalog(profile, &response)?;
    Ok(ModelCatalog {
        models,
        source: format!("{} {:?}", profile.executable, discovery.args),
        queried_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        cli_version: "custom".into(),
        origin: CatalogOrigin::Discovered,
    })
}
