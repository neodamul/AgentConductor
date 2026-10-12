use super::*;
impl Startup {
    pub(super) fn launch_quota(
        &self,
        saved: &mut Saved,
        agent: &str,
    ) -> Result<serde_json::Value, String> {
        let profile = saved
            .config
            .profiles
            .iter()
            .find(|p| p.id == agent)
            .ok_or("unknown agent")?;
        if !matches!(
            profile.parser,
            crate::protocol::Provider::Claude | crate::protocol::Provider::Codex
        ) {
            return Err("quota query is unsupported for this agent".into());
        }
        if self.quota_job(saved)?["state"] == "running" {
            return Err("quota refresh already running".into());
        }
        fs::create_dir_all(&self.quota_broker.dir).map_err(|e| e.to_string())?;
        let mut slot = self
            .quota_broker
            .job
            .lock()
            .map_err(|_| "quota job state poisoned")?;
        let operation = format!("quota:{agent}");
        let args = vec![
            "workspace-quota".into(),
            self.broker.dir.to_string_lossy().into(),
            agent.into(),
        ];
        let result = self.quota_broker.launch_locked(
            &operation,
            &args,
            &self.revision(saved)?,
            &mut slot,
        )?;
        saved.quota_job = Some(StoredJob {
            id: result["id"].as_str().ok_or("missing quota job ID")?.into(),
            operation,
            started_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| "clock out of range")?,
        });
        self.store(saved)?;
        Ok(result)
    }

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
                        serde_json::json!({"status":"unavailable","windows":[],"error":"No retained account quota observation"})
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
