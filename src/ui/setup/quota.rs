use super::*;
impl Startup {
    pub(super) fn quota_snapshot(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        let run = self.last_run(saved);
        let store = run
            .as_ref()
            .filter(|p| p.join("state.sqlite").is_file())
            .map(|p| crate::state::Store::read_only_snapshot(p.join("state.sqlite")))
            .transpose()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let workspace = self.broker.dir.join("quota.sqlite");
        let workspace = crate::quota::workspace_cache(&workspace)?;
        let mut quotas = serde_json::Map::new();
        for profile in &saved.config.profiles {
            if matches!(
                profile.parser,
                crate::protocol::Provider::Claude | crate::protocol::Provider::Codex
            ) {
                let value = match &store {
                    Some(store) => crate::quota::stored(store, profile, now)?,
                    None => {
                        serde_json::json!({"status":"unavailable","windows":[],"error":crate::ui::messages::text("native.no_retained_account_quota_observation")})
                    }
                };
                let local = workspace
                    .as_ref()
                    .map(|store| crate::quota::stored(store, profile, now))
                    .transpose()?;
                let value = match local {
                    Some(local)
                        if local["observed_at"].as_i64().unwrap_or(0)
                            >= value["observed_at"].as_i64().unwrap_or(0) =>
                    {
                        local
                    }
                    _ => value,
                };
                quotas.insert(profile.id.clone(), value);
            }
        }
        Ok(quotas.into())
    }
}
