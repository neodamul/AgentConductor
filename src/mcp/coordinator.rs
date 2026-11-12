//! #898: a bounded CLI allowance shared by MCP selection and summaries.
use super::Reference;
use crate::{plan::AgentProfile, ui::messages};
use serde_json::Value;
use std::{path::Path, time::Duration};
pub(super) const STEP_TIMEOUT: Duration = Duration::from_secs(300);
fn step_error(error: String, field: &str, reference: &Reference, dir: &Path) -> String {
    if !error.starts_with("coordinator failed (timed out)") {
        return error;
    }
    let stage = messages::text(if field == "selection" {
        "mcp.selection_step"
    } else {
        "mcp.summary_step"
    });
    format!(
        "{}\nlogs: {}",
        messages::render(
            "mcp.coordinator_step_timed_out",
            &[
                ("reference", &reference.key()),
                ("stage", stage),
                ("seconds", &STEP_TIMEOUT.as_secs().to_string())
            ]
        ),
        dir.join("process").display()
    )
}
pub(super) async fn run_step(
    profile: &AgentProfile,
    root: &Path,
    dir: &Path,
    prompt: &str,
    field: &str,
    reference: &Reference,
    timeout: Duration,
) -> Result<Value, String> {
    if timeout.is_zero() {
        return Err(step_error(
            "coordinator failed (timed out)".into(),
            field,
            reference,
            dir,
        ));
    }
    // Capacity/format retries share this deadline; cancellation drops the owned process guard.
    match tokio::time::timeout(
        timeout,
        crate::coordinator::run_json_within(profile, root, dir, prompt, field, None, timeout),
    )
    .await
    {
        Ok(result) => result.map_err(|error| step_error(error, field, reference, dir)),
        Err(_) => Err(step_error(
            "coordinator failed (timed out)".into(),
            field,
            reference,
            dir,
        )),
    }
}

/// The full schema remains local evidence; bounded schemas go directly in the next CLI input.
pub(super) fn observe_schema(
    catalog: &mut Vec<Value>,
    observations: &mut Vec<Value>,
    server: &str,
    tool: &Value,
    root: &Path,
    goal: &str,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(tool).map_err(|e| e.to_string())?;
    let name = format!("schema-{}.json", crate::workflow::digest(&bytes));
    super::imports::store(root, &name, &bytes)?;
    *catalog=catalog.iter().map(|t|serde_json::json!({"server":t["server"],"name":t["name"],"description":t["description"]})).collect();
    for previous in observations.iter_mut() {
        if let Some(object) = previous.as_object_mut() {
            object.remove("schema");
        }
    }
    let canonical = root
        .join("mcp")
        .join(name)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let mut observation = serde_json::json!({"server":server,"tool":tool["name"],"schema_path":std::path::PathBuf::from(crate::git::git_path(canonical.as_os_str()))});
    let used = serde_json::to_vec(&(&catalog, &observations, goal))
        .map_err(|e| e.to_string())?
        .len();
    if used.saturating_add(bytes.len()) <= 96 * 1024 {
        observation.as_object_mut().unwrap().remove("schema_path");
        observation["schema"] = tool.clone();
    }
    observations.push(observation);
    Ok(())
}
#[cfg(test)]
#[path = "../../tests/support/mcp_step_error.rs"]
mod tests;
