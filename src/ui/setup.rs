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
            model_discovery: None,
            execution_options: vec![],
            result_pointer: None,
            error_condition: None,
            max_sessions: 3,
            quota_refresh_minutes: 5,
            quota_group: None,
            fallback: None,
            cooperative: Default::default(),
            event_types: Default::default(),
            session_id_pointer: None,
            quota_command: None,
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
mod storage;
use storage::{read, write};

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
mod runtime;

mod automatic;
