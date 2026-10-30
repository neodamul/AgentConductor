use super::{automatic, metadata, profiles};
use crate::{
    adapters,
    plan::{AgentProfile, WorkspaceMode},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
pub struct Proposal {
    #[serde(default)]
    pub(super) workspace: Option<PathBuf>,
    pub(super) directory: PathBuf,
    pub(super) repository_base: String,
    pub(super) run: PathBuf,
    pub(super) phase: String,
    /// DD-62: the slug and phase this proposal executes, when phased.
    #[serde(default)]
    pub(super) slug: Option<String>,
    #[serde(default)]
    pub(super) slug_phase: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct StoredJob {
    pub(super) id: String,
    pub(super) operation: String,
    /// DD-53: launch time so every window shows the true elapsed time.
    #[serde(default)]
    pub(super) started_ms: u64,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Saved {
    #[serde(default)]
    pub(super) imports: Option<crate::mcp::Applied>,
    #[serde(default)]
    pub(super) mcp_result: Option<String>,
    #[serde(default)]
    pub(super) chat_result: Option<String>,
    #[serde(default)]
    pub(super) optimize_result: Option<String>,
    pub(super) config: Config,
    pub(super) proposal: Option<Proposal>,
    pub(super) job: Option<StoredJob>,
    #[serde(default)]
    pub(super) quota_job: Option<StoredJob>,
    /// GitHub #261 (DD-204): the coordinator chat has its own slot, so it runs next to a plan or start job.
    #[serde(default)]
    pub(super) chat_job: Option<StoredJob>,
    #[serde(default)]
    pub(super) metadata_queue: Vec<metadata::Pending>,
    #[serde(default)]
    pub(super) metadata_input: Option<PathBuf>,
    #[serde(default)]
    pub(super) metadata_settled: Option<String>,
    #[serde(default)]
    pub(super) metadata_error: Option<serde_json::Value>,
    /// Most recently started run, kept when the editable proposal changes (DD-39).
    #[serde(default)]
    pub(super) last_run: Option<PathBuf>,
}

pub fn defaults() -> Result<Config, String> {
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
            protocol: None,
            agent_settings: None,
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
