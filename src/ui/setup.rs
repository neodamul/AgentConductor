use super::Controller;
use crate::ui::messages;
use crate::{
    adapters,
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "automatic::enabled")]
    pub auto_resume: bool,
    #[serde(default)]
    pub mcp: std::collections::BTreeMap<String, crate::mcp::Server>,
    #[serde(default)]
    pub goal_sources: std::collections::BTreeMap<String, crate::mcp::Binding>,
    pub profiles: Vec<AgentProfile>,
    pub repository: PathBuf,
    pub goal: String,
    pub coordinator: String,
    #[serde(default)]
    pub workspace_mode: WorkspaceMode,
    /// DD-48: operator-visible workspace name; empty falls back to the folder name.
    #[serde(default)]
    pub name: String,
    /// DD-55: who performs which role; sent to the planning coordinator.
    #[serde(default)]
    pub roles: crate::roles::RoleRules,
    /// DD-61: every goal pursued here, oldest first, and the one being worked on.
    #[serde(default)]
    pub slugs: Vec<crate::slugs::Slug>,
    #[serde(default)]
    pub active_slug: Option<String>,
    /// DD-65: workspace plan rules; `None` uses the global rules file.
    #[serde(default)]
    pub plan_rules: Option<String>,
    /// DD-79: messenger notifications; the webhook URL is stored outside the workspace.
    #[serde(default)]
    pub messenger: crate::messenger::Settings,
}
#[derive(Clone, Serialize, Deserialize)]
struct Proposal {
    #[serde(default)]
    workspace: Option<PathBuf>,
    directory: PathBuf,
    repository_base: String,
    run: PathBuf,
    phase: String,
    /// DD-62: the slug and phase this proposal executes, when phased.
    #[serde(default)]
    slug: Option<String>,
    #[serde(default)]
    slug_phase: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
struct StoredJob {
    id: String,
    operation: String,
    /// DD-53: launch time so every window shows the true elapsed time.
    #[serde(default)]
    started_ms: u64,
}
#[derive(Clone, Serialize, Deserialize)]
struct Saved {
    #[serde(default)]
    imports: Option<crate::mcp::Applied>,
    #[serde(default)]
    mcp_result: Option<String>,
    config: Config,
    proposal: Option<Proposal>,
    job: Option<StoredJob>,
    #[serde(default)]
    quota_job: Option<StoredJob>,
    #[serde(default)]
    metadata_queue: Vec<metadata::Pending>,
    #[serde(default)]
    metadata_input: Option<PathBuf>,
    #[serde(default)]
    metadata_settled: Option<String>,
    #[serde(default)]
    metadata_error: Option<serde_json::Value>,
    /// Most recently started run, kept when the editable proposal changes (DD-39).
    #[serde(default)]
    last_run: Option<PathBuf>,
}

/// Local startup state. Only explicit native requests launch a trusted CLI child.
pub struct Startup {
    broker: Controller,
    quota_broker: Controller,
    saved: Mutex<Saved>,
    custom_profiles: Vec<AgentProfile>,
    custom_registry_invalid: bool,
    _lease: crate::leases::Lease,
}
fn defaults() -> Result<Config, String> {
    let mut profiles = Vec::new();
    for (id, provider) in [
        ("claude", crate::protocol::Provider::Claude),
        ("codex", crate::protocol::Provider::Codex),
        ("opencode", crate::protocol::Provider::OpenCode),
    ] {
        let invocation = adapters::invocation(provider, None, "{task_file}")?;
        profiles.push(AgentProfile {
            id: id.into(),
            executable: crate::executable::resolve(&invocation.executable)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or(invocation.executable),
            args: invocation.args,
            parser: provider,
            input_mode: invocation.input,
            environment: Default::default(),
            permission_mode: "unverified".into(),
            catalog: None,
            max_sessions: 3,
            quota_refresh_minutes: 5,
            quota_group: None,
            fallback: None,
            cooperative: Default::default(),
            event_types: Default::default(),
            session_id_pointer: None,
        });
    }
    profiles.iter_mut().for_each(profiles::derive_permission);
    Ok(Config {
        auto_resume: true,
        mcp: Default::default(),
        goal_sources: Default::default(),
        profiles,
        repository: PathBuf::new(),
        goal: String::new(),
        name: String::new(),
        roles: Default::default(),
        slugs: vec![],
        active_slug: None,
        plan_rules: None,
        messenger: Default::default(),
        coordinator: "codex".into(),
        workspace_mode: WorkspaceMode::Shared,
    })
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
    fn last_run(&self, saved: &Saved) -> Option<PathBuf> {
        let run = saved.last_run.as_ref()?.canonicalize().ok()?;
        let root = self.broker.dir.canonicalize().ok()?;
        (run.starts_with(&root) && run.join("report.json").is_file()).then_some(run)
    }
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
