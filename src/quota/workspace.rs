use super::*;
use std::path::Path;
/// A run can use metadata from its explicitly bound parent workspace, never an arbitrary ancestor.
pub fn stored_for_run(
    store: &crate::state::Store,
    profile: &crate::plan::AgentProfile,
    now: i64,
    run: &Path,
) -> Result<Value, String> {
    let current = stored(store, profile, now)?;
    let Some(parent) = run.parent() else {
        return Ok(current);
    };
    let settings = parent.join("startup.json");
    let cache = parent.join("quota.sqlite");
    if !settings.is_file() || !cache.is_file() {
        return Ok(current);
    }
    let bytes = std::fs::read(settings).map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Ok(current);
    }
    let saved: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let canonical = run.canonicalize().map_err(|e| e.to_string())?;
    let bound = [
        saved["last_run"].as_str(),
        saved["proposal"]["run"].as_str(),
    ]
    .into_iter()
    .flatten()
    .any(|p| Path::new(p).canonicalize().ok().as_ref() == Some(&canonical));
    if !bound {
        return Ok(current);
    }
    let cache = crate::state::Store::read_only_snapshot(cache)?;
    let value = stored(&cache, profile, now)?;
    Ok(
        if value["observed_at"].as_i64().unwrap_or(0) > current["observed_at"].as_i64().unwrap_or(0)
        {
            value
        } else {
            current
        },
    )
}
