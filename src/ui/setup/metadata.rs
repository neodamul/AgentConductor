//! One bounded metadata worker, with a small durable queue; no planning-slot lock.
use super::*;
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Pending {
    pub kind: String,
    pub agent: String,
}
impl Startup {
    pub(super) fn enqueue_metadata(
        &self,
        saved: &mut Saved,
        kind: &str,
        agent: &str,
    ) -> Result<serde_json::Value, String> {
        let profile = saved
            .config
            .profiles
            .iter()
            .find(|p| p.id == agent)
            .ok_or("unknown agent")?;
        match kind {
            "quota"
                if !matches!(
                    profile.parser,
                    crate::protocol::Provider::Claude | crate::protocol::Provider::Codex
                ) =>
            {
                return Err("quota query is unsupported for this agent".into());
            }
            "catalog" if profile.parser == crate::protocol::Provider::Custom => {
                return Err("custom agents import an observed catalog".into());
            }
            "quota" | "catalog" => (),
            _ => return Err("unknown metadata query".into()),
        }
        let operation = format!("{kind}:{agent}");
        let job = self.quota_job(saved)?;
        if job["state"] == "running" {
            if job["kind"] == operation {
                return Err("metadata refresh already running".into());
            }
            if !saved
                .metadata_queue
                .iter()
                .any(|p| p.kind == kind && p.agent == agent)
            {
                if saved.metadata_queue.len() >= 32 {
                    return Err("metadata queue is full".into());
                }
                saved.metadata_queue.push(Pending {
                    kind: kind.into(),
                    agent: agent.into(),
                });
                self.store(saved)?;
            }
            return Ok(serde_json::json!({"queued":true}));
        }
        self.launch_metadata(
            saved,
            Pending {
                kind: kind.into(),
                agent: agent.into(),
            },
        )
    }
    fn launch_metadata(
        &self,
        saved: &mut Saved,
        pending: Pending,
    ) -> Result<serde_json::Value, String> {
        fs::create_dir_all(&self.quota_broker.dir).map_err(|e| e.to_string())?;
        let mut slot = self
            .quota_broker
            .job
            .lock()
            .map_err(|_| "metadata job state poisoned")?;
        let operation = format!("{}:{}", pending.kind, pending.agent);
        let mut input = None;
        let args = if pending.kind == "catalog" {
            let folder = self
                .quota_broker
                .dir
                .join(format!("catalog-{}", crate::native_guard::session_id()?));
            fs::create_dir(&folder).map_err(|e| e.to_string())?;
            let path = folder.join("profiles.json");
            write(&path, &saved.config.profiles)?;
            let args = vec![
                "models-discover".into(),
                path.to_string_lossy().into(),
                pending.agent,
                folder.join("discovery").to_string_lossy().into(),
            ];
            input = Some(path);
            args
        } else {
            vec![
                "workspace-quota".into(),
                self.broker.dir.to_string_lossy().into(),
                pending.agent,
            ]
        };
        let result = self.quota_broker.launch_locked(
            &operation,
            &args,
            &self.revision(saved)?,
            &mut slot,
        )?;
        saved.quota_job = Some(StoredJob {
            id: result["id"]
                .as_str()
                .ok_or("missing metadata job ID")?
                .into(),
            operation,
            started_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| "clock out of range")?,
        });
        saved.metadata_input = input;
        saved.metadata_settled = None;
        saved.metadata_error = None;
        self.store(saved)?;
        Ok(result)
    }
    pub(super) fn metadata_tick(&self, saved: &mut Saved) -> Result<serde_json::Value, String> {
        let live = self
            .quota_broker
            .job
            .lock()
            .map(|slot| slot.as_ref().is_some_and(|j| !j.finished))
            .unwrap_or(true);
        if !live
            && saved.metadata_queue.is_empty()
            && let Some(error) = &saved.metadata_error
            && error["id"].as_str() == saved.metadata_settled.as_deref()
        {
            return Ok(error.clone());
        }
        match self.metadata_step(saved) {
            Ok(job) => {
                if job["state"] != "running"
                    && let Some(error) = &saved.metadata_error
                    && job["id"] == error["id"]
                {
                    return Ok(error.clone());
                }
                Ok(job)
            }
            Err(error) => {
                let live = self
                    .quota_broker
                    .job
                    .lock()
                    .map(|slot| slot.as_ref().is_some_and(|j| !j.finished))
                    .unwrap_or(true);
                let id = saved.quota_job.as_ref().map(|j| j.id.clone());
                let mut failure = serde_json::json!({"id":id,"kind":saved.quota_job.as_ref().map(|j|j.operation.clone()),"state":if live {"running"} else {"failed"},"error":error});
                if !live {
                    saved.metadata_settled = id;
                }
                saved.metadata_error = Some(failure.clone());
                let _ = crate::state::Store::open(self.broker.dir.join("state.sqlite")).and_then(
                    |mut store| {
                        store.record(
                            "metadata_failure",
                            &failure,
                            "Metadata failed without blocking workspace controls",
                        )
                    },
                );
                if let Err(error) = self.store(saved) {
                    failure["error"] = format!(
                        "{}; cannot persist metadata failure: {error}",
                        failure["error"].as_str().unwrap_or("metadata failure")
                    )
                    .into();
                    saved.metadata_error = Some(failure.clone());
                }
                Ok(failure)
            }
        }
    }
    fn metadata_step(&self, saved: &mut Saved) -> Result<serde_json::Value, String> {
        let mut job = self.quota_job(saved)?;
        if job["state"] == "running" {
            return Ok(job);
        }
        if let Some(stored) = saved.quota_job.clone()
            && saved.metadata_settled.as_ref() != Some(&stored.id)
        {
            if stored.operation.starts_with("catalog:") && job["state"] == "completed" {
                let observed = saved
                    .metadata_input
                    .as_ref()
                    .ok_or_else(|| "model query input missing".to_string())
                    .and_then(|path| read::<Vec<AgentProfile>>(path));
                match observed {
                    Ok(profiles) => {
                        let agent = stored.operation.trim_start_matches("catalog:");
                        if let Some(current) =
                            saved.config.profiles.iter_mut().find(|p| p.id == agent)
                            && let Some(observed) = profiles.iter().find(|p| p.id == agent)
                            && profiles::execution_view(std::slice::from_ref(current))?
                                == profiles::execution_view(std::slice::from_ref(observed))?
                        {
                            if let Some(catalog) = &observed.catalog {
                                validate_catalog(catalog)?;
                            }
                            current.catalog = observed.catalog.clone();
                        }
                    }
                    Err(_) => {
                        job["state"] = "failed".into();
                        job["error"] =
                            "model query result unavailable; inspect retained metadata logs".into();
                    }
                }
            }
            saved.metadata_settled = Some(stored.id);
            self.store(saved)?;
        }
        while !saved.metadata_queue.is_empty() {
            let next = saved.metadata_queue.remove(0);
            if !saved.config.profiles.iter().any(|p| p.id == next.agent) {
                self.store(saved)?;
                continue;
            }
            let operation = format!("{}:{}", next.kind, next.agent);
            self.launch_metadata(saved, next)
                .map_err(|e| format!("{operation} launch failed: {e}"))?;
            return self.quota_job(saved);
        }
        Ok(job)
    }
}
