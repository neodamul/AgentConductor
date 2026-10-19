use super::Snapshot;
use crate::{plan::AgentProfile, process::ProcessSpec};
use serde_json::Value;
use std::{path::Path, time::Duration};

pub fn parse_custom_output(raw: &Value, pointer: Option<&str>) -> Result<Snapshot, String> {
    let value = match pointer {
        Some("") => raw,
        Some(pointer) => raw
            .pointer(pointer)
            .ok_or("quota output JSON Pointer was not found")?,
        None => raw,
    };
    let snapshot: Snapshot = serde_json::from_value(value.clone())
        .map_err(|_| "custom quota output must match the normalized Snapshot JSON schema")?;
    if snapshot.source.is_empty()
        || snapshot.source.len() > 256
        || snapshot.observed_at <= 0
        || snapshot.windows.len() > 64
        || snapshot.budgets.len() > 64
        || snapshot.windows.iter().any(|window| {
            window.bucket.is_empty()
                || window.bucket.len() > 256
                || window.label.is_empty()
                || window.label.len() > 256
                || window
                    .used_percent
                    .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                || window
                    .remaining_percent
                    .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                || window.window_minutes == Some(0)
                || window.resets_at.is_some_and(|at| at <= 0)
        })
        || snapshot.budgets.iter().any(|budget| {
            budget.bucket.is_empty()
                || budget.bucket.len() > 256
                || budget.scope.is_empty()
                || budget.scope.len() > 256
                || budget
                    .remaining_percent
                    .is_some_and(|n| !n.is_finite() || !(0.0..=100.0).contains(&n))
                || budget.resets_at.is_some_and(|at| at <= 0)
        })
    {
        return Err("custom quota output contains invalid or excessive values".into());
    }
    Ok(snapshot)
}

pub async fn query_custom(profile: &AgentProfile, dir: &Path) -> Result<Snapshot, String> {
    let config = profile
        .quota_command
        .as_ref()
        .ok_or("custom CLI quota command is not configured")?;
    profile.validate_custom_config()?;
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    let executable = crate::executable::resolve(&profile.executable)
        .unwrap_or_else(|| profile.executable.clone().into());
    let args = config
        .args
        .iter()
        .map(|arg| {
            arg.replace("{workspace}", &dir.to_string_lossy())
                .replace("{agent_id}", &profile.id)
        })
        .collect();
    let result = crate::process::run(ProcessSpec {
        executable,
        args,
        cwd: dir.clone(),
        input: vec![],
        log_dir: dir.join("process"),
        timeout: Duration::from_secs(u64::from(config.timeout_seconds)),
        signal_grace: Duration::from_secs(3),
        environment: profile.environment.clone(),
    })
    .await?;
    if result.timed_out {
        return Err("custom quota command timed out; inspect retained process logs".into());
    }
    if result.exit_code != Some(0) {
        return Err(format!(
            "custom quota command exited with {}; inspect retained process logs",
            result
                .exit_code
                .map_or_else(|| "no code".into(), |code| code.to_string())
        ));
    }
    let output = tokio::fs::read(&result.stdout)
        .await
        .map_err(|e| e.to_string())?;
    if output.len() > 1024 * 1024 {
        return Err("custom quota output exceeds 1 MiB".into());
    }
    let raw: Value = serde_json::from_slice(&output)
        .map_err(|_| "custom quota command did not return one JSON object")?;
    parse_custom_output(&raw, config.output_pointer.as_deref())
}
