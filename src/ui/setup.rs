use super::Controller;
use crate::ui::messages;
use crate::{
    git::Repository,
    plan::{AgentProfile, ModelCatalog, Plan, Task, WorkspaceMode},
    workflow,
};
mod model;
pub use model::{Config, Proposal, Saved, StoredJob};
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
    heads: crate::git_head::Heads,
    saved: Mutex<Saved>,
    custom_profiles: Vec<AgentProfile>,
    custom_registry_invalid: bool,
    _lease: crate::leases::Lease,
}
fn defaults() -> Result<Config, String> {
    model::defaults()
}
mod startup;
mod storage;
use storage::{bytes, read, write};

impl Startup {
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

mod baseline;
mod chat;
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
mod optimize;
mod project;
mod quota;
mod run_binding;

mod automatic;
