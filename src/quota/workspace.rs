use super::*;
use std::path::Path;
/// Metadata writers create the file before publishing the schema; this is not an error.
pub(crate) fn workspace_cache(path: &Path) -> Result<Option<crate::state::Store>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let store = crate::state::Store::read_only_snapshot(path)?;
    let ready: bool = store
        .0
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='documents')",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(ready.then_some(store))
}
/// A run can use metadata from its explicitly bound parent workspace, never an arbitrary ancestor.
pub fn stored_for_run(
    store: &crate::state::Store,
    profile: &crate::plan::AgentProfile,
    now: i64,
    run: &Path,
) -> Result<Value, String> {
    let mut current = stored(store, profile, now)?;
    let Some(parent) = run.parent() else {
        return Ok(current);
    };
    let settings = parent.join("startup.json");
    let cache = parent.join("quota.sqlite");
    if !settings.is_file() {
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
    let minutes = saved["config"]["profiles"]
        .as_array()
        .and_then(|agents| {
            agents.iter().find_map(|value| {
                let candidate: crate::plan::AgentProfile =
                    serde_json::from_value(value.clone()).ok()?;
                (candidate.id == profile.id
                    && profile_digest(&candidate).ok()? == profile_digest(profile).ok()?)
                .then_some(candidate.quota_refresh_minutes)
            })
        })
        .unwrap_or(profile.quota_refresh_minutes);
    current["refresh_minutes"] = minutes.into();
    let Some(cache) = workspace_cache(&cache)? else {
        return Ok(current);
    };
    let mut value = stored(&cache, profile, now)?;
    value["refresh_minutes"] = minutes.into();
    Ok(
        if value["observed_at"].as_i64().unwrap_or(0) > current["observed_at"].as_i64().unwrap_or(0)
        {
            value
        } else {
            current
        },
    )
}
