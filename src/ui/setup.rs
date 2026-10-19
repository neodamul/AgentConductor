use super::Controller;
use crate::ui::messages;
use crate::{
    git::Repository,
    plan::{AgentProfile, ModelCatalog, Plan, Task, WorkspaceMode},
    workflow,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

/// Local startup state. Only explicit native requests launch a trusted CLI child.
pub struct Startup {
    broker: Controller,
    quota_broker: Controller,
    saved: Mutex<Saved>,
    _lease: crate::leases::Lease,
}
mod model;
pub use model::Config;
pub(super) use model::{Proposal, Saved, StoredJob, defaults};

mod settle;
mod storage;
use storage::{bytes, read, write};

impl Startup {
    pub fn open(root: &Path, exe: &Path) -> Result<Self, String> {
        if !root.exists() {
            fs::create_dir_all(root).map_err(|e| e.to_string())?;
        }
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
        }
        let lease = crate::leases::Lease::acquire(&root.join("startup.lock"))?;
        let settings = root.join("startup.json");
        let mut saved: Saved = if settings.exists() {
            read(&settings)?
        } else {
            let s = Saved {
                imports: None,
                mcp_result: None,
                // DD-72: no pre-registered agents; templates stay available in + Add.
                config: Config {
                    profiles: vec![],
                    coordinator: String::new(),
                    ..defaults()?
                },
                proposal: None,
                job: None,
                quota_job: None,
                metadata_queue: Vec::new(),
                metadata_input: None,
                metadata_settled: None,
                metadata_error: None,
                last_run: None,
            };
            write(&settings, &s)?;
            s
        };
        master::adopt(&root, &mut saved.config.profiles);
        saved.last_run.as_deref().map(crate::workflow::sync_phase);
        // A reviewed plan whose private folder copy is gone (for example after the legacy
        // folder-history copies were removed) can no longer start; it is finished history.
        if saved
            .proposal
            .as_ref()
            .and_then(|p| p.workspace.as_ref())
            .is_some_and(|w| !w.exists())
        {
            saved.proposal = None;
            write(&settings, &saved)?;
        }
        let startup = Self {
            quota_broker: Controller {
                dir: root.join("quota-jobs"),
                exe: exe.canonicalize().map_err(|e| e.to_string())?,
                job: Mutex::new(None),
            },
            broker: Controller {
                dir: root,
                exe: exe.canonicalize().map_err(|e| e.to_string())?,
                job: Mutex::new(None),
            },
            saved: Mutex::new(saved),
            _lease: lease,
        };
        if let Ok(saved) = startup.saved.lock() {
            let _ = startup.clear_cache(&saved, None, 8);
            let _ = startup.prune_history(&saved);
        }
        startup.recover_last_run()?;
        Ok(startup)
    }
    fn revision(&self, s: &Saved) -> Result<String, String> {
        // Observations and scheduler progress cannot invalidate an unrelated edit.
        let mut context = serde_json::to_value(s).map_err(|e| e.to_string())?;
        for key in [
            "quota_job",
            "metadata_queue",
            "metadata_input",
            "metadata_settled",
            "metadata_error",
        ] {
            context.as_object_mut().unwrap().remove(key);
        }
        if let Some(profiles) = context["config"]["profiles"].as_array_mut() {
            for profile in profiles {
                profile.as_object_mut().unwrap().remove("catalog");
            }
        }
        let mut bytes = serde_json::to_vec(&context).map_err(|e| e.to_string())?;
        if let Some(p) = &s.proposal {
            let path = p.directory.join("review.json");
            if path.exists() {
                bytes.extend(self::bytes(&path)?);
            }
            bytes.extend(
                Repository::open(p.workspace.as_deref().unwrap_or(&s.config.repository))?
                    .head()?
                    .as_bytes(),
            );
        }
        Ok(workflow::digest(&bytes))
    }
    fn quota_job(&self, s: &Saved) -> Result<serde_json::Value, String> {
        jobs::state(&self.quota_broker, &s.quota_job)
    }

    fn launch(
        &self,
        s: &mut Saved,
        operation: &str,
        args: Vec<String>,
    ) -> Result<serde_json::Value, String> {
        let mut slot = self
            .broker
            .job
            .lock()
            .map_err(|_| messages::text("native.startup_job_state_poisoned"))?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err(messages::text("native.startup_operation_still_running").into());
        }
        let result = self
            .broker
            .launch_locked(operation, &args, &self.revision(s)?, &mut slot)?;
        s.job = Some(StoredJob {
            id: result["id"]
                .as_str()
                .ok_or(messages::text("native.missing_job_id"))?
                .into(),
            operation: operation.into(),
            started_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| messages::text("native.clock_out_of_range"))?,
        });
        self.store(s)?;
        Ok(result)
    }
    pub(crate) fn poll_job(&self) -> Result<(), String> {
        self.broker.poll()?;
        self.quota_broker.poll().map(|_| ())
    }
    pub(crate) fn root(&self) -> &Path {
        &self.broker.dir
    }
    /// DD-48: name a freshly created workspace through the same validated store.
    pub(crate) fn set_name(&self, name: &str) -> Result<(), String> {
        validate_name(name)?;
        let mut saved = self
            .saved
            .lock()
            .map_err(|_| messages::text("native.startup_settings_poisoned"))?;
        let mut next = saved.clone();
        next.config.name = name.into();
        self.store(&next)?;
        *saved = next;
        Ok(())
    }
    /// Switching workspaces must not orphan a planning or start job.
    pub(crate) fn ensure_idle(&self) -> Result<(), String> {
        let saved = self
            .saved
            .lock()
            .map_err(|_| messages::text("native.startup_settings_poisoned"))?;
        if self.job(&saved)?["state"] == "running" || self.quota_job(&saved)?["state"] == "running"
        {
            return Err(
                messages::text("native.wait_for_the_running_planning_or_metadata_job").into(),
            );
        }
        drop(saved);
        self.ensure_no_live_jobs()
    }
    pub(crate) fn executable(&self) -> &Path {
        &self.broker.exe
    }
    pub(crate) fn active_run(&self) -> Result<Option<PathBuf>, String> {
        let saved = self
            .saved
            .lock()
            .map_err(|_| messages::text("native.startup_settings_poisoned"))?;
        if let Some(proposal) = &saved.proposal
            && proposal.phase == "starting"
            && proposal.run.join("report.json").exists()
        {
            let report = workflow::load(&proposal.run)?;
            if report.plan.goal != saved.config.goal
                || !run_binding::matches(
                    proposal
                        .workspace
                        .as_deref()
                        .unwrap_or(&saved.config.repository),
                    &saved.config.repository,
                    &proposal.repository_base,
                    &report.repo,
                )?
                || report.base != proposal.repository_base
            {
                return Err(
                    messages::text("native.run_does_not_belong_to_the_reviewed_startup").into(),
                );
            }
            return Ok(Some(proposal.run.clone()));
        }
        Ok(self.last_run(&saved))
    }
    /// The started run survives later draft edits, but only inside this workspace.
    fn ensure_no_live_jobs(&self) -> Result<(), String> {
        for entry in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry
                .file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
            {
                continue;
            }
            let path = entry.path().join("process/owner.json");
            let owner: crate::process::Ownership = read(&path)?;
            // Successful startup commands have reaped their workers; old PIDs can be reused.
            if owner.phase == "terminal"
                && owner.exit_code == Some(0)
                && owner.pid.is_some_and(|id| id > 0 && id <= i32::MAX as u32)
                && owner.process_group == owner.pid.map(|id| id as i32)
            {
                continue;
            }
            owner.ensure_quiescent().map_err(|e| {
                messages::render(
                    "native.retained_startup_job_is_live_or_unknown",
                    &[("e", &e.to_string())],
                )
            })?;
        }
        Ok(())
    }
}

mod document;
pub(crate) use document::document;

mod baseline;
mod messenger;
mod request;
#[cfg(test)]
#[path = "../../tests/unit/startup.rs"]
mod tests;
use request::Request;
mod dispatch;
mod planning;
mod profiles;
mod share;
mod slug;
mod snapshot;
pub(crate) use profiles::{validate_catalog, validate_name, validate_profiles};

mod jobs;
mod lifecycle;
mod master;
mod mcp;
mod metadata;
mod project;
mod quota;
mod run_binding;

mod automatic;
