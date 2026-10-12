use super::Controller;
use crate::{
    adapters,
    git::Repository,
    plan::{AgentProfile, ModelCatalog, Plan, Task, WorkspaceMode},
    workflow,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
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
            permission_mode: if id == "codex" {
                "native_sandbox"
            } else {
                "unverified"
            }
            .into(),
            catalog: None,
            max_sessions: 3,
            quota_refresh_minutes: 5,
            quota_group: None,
            fallback: None,
            cooperative: Default::default(),
        });
    }
    Ok(Config {
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
fn bytes(path: &Path) -> Result<Vec<u8>, String> {
    let file = super::open_artifact(path)?;
    let mut data = Vec::new();
    file.take(65537)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 65536 {
        return Err("startup document exceeds 64 KiB".into());
    }
    Ok(data)
}
fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&bytes(path)?).map_err(|e| e.to_string())
}
fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if path.exists() || fs::symlink_metadata(path).is_ok() {
        super::open_artifact(path)?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if bytes.len() > 64 * 1024 {
        return Err("startup document exceeds 64 KiB".into());
    }
    let dir = path.parent().ok_or("missing settings directory")?;
    let pending = dir.join(format!("{}.pending", crate::native_guard::session_id()?));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    fs::rename(&pending, path).map_err(|e| e.to_string())?;
    crate::process::sync_directory(dir)
}
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
        let saved = if settings.exists() {
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
                last_run: None,
            };
            write(&settings, &s)?;
            s
        };
        Ok(Self {
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
        })
    }
    fn revision(&self, s: &Saved) -> Result<String, String> {
        let mut bytes = serde_json::to_vec(s).map_err(|e| e.to_string())?;
        if let Some(p) = &s.proposal {
            let path = p.directory.join("review.json");
            if path.exists() {
                bytes.extend(self::bytes(&path)?);
            }
            bytes.extend(Repository::open(&s.config.repository)?.head()?.as_bytes());
        }
        Ok(workflow::digest(&bytes))
    }
    fn store(&self, s: &Saved) -> Result<(), String> {
        write(&self.broker.dir.join("startup.json"), s)
    }
    fn job(&self, s: &Saved) -> Result<serde_json::Value, String> {
        jobs::state(&self.broker, &s.job)
    }
    fn quota_job(&self, s: &Saved) -> Result<serde_json::Value, String> {
        jobs::state(&self.quota_broker, &s.quota_job)
    }
    fn settle(&self, s: &mut Saved, job: &serde_json::Value) -> Result<(), String> {
        if job["state"] != "completed" {
            return Ok(());
        }
        if s.job.as_ref().is_some_and(|j| j.operation == "plan")
            && s.proposal.as_ref().is_some_and(|p| p.phase == "planning")
        {
            let p = s.proposal.as_mut().unwrap();
            let mut plan: Plan = read(&p.directory.join("plan.json"))?;
            if plan.goal != s.config.goal
                || serde_json::to_vec(&plan.agents).map_err(|e| e.to_string())?
                    != serde_json::to_vec(&s.config.profiles).map_err(|e| e.to_string())?
            {
                return Err("planner changed registered profiles or goal".into());
            }
            plan.workspace_mode = s.config.workspace_mode;
            plan.validate()?;
            write(&p.directory.join("review.json"), &plan)?;
            p.phase = "ready".into();
            crate::messenger::notify_plan(
                &self.broker.dir,
                &s.config.messenger,
                &s.config.name,
                &s.config.goal,
                plan.tasks.len(),
            );
            if let Some(slug) = &p.slug {
                let root = crate::slugs::root(&s.config.repository, &self.broker.dir, slug);
                p.slug_phase = crate::slug_progress::progress(&root)["current_phase"]
                    .as_str()
                    .map(str::to_owned);
            }
            self.store(s)?;
        } else if s
            .job
            .as_ref()
            .is_some_and(|j| j.operation.starts_with("catalog:"))
        {
            let profiles: Vec<AgentProfile> = read(&self.broker.dir.join("profiles.json"))?;
            validate_profiles(&profiles)?;
            let mut expected = s.config.profiles.clone();
            for (a, b) in expected.iter_mut().zip(&profiles) {
                a.catalog = b.catalog.clone();
            }
            if serde_json::to_vec(&expected).map_err(|e| e.to_string())?
                != serde_json::to_vec(&profiles).map_err(|e| e.to_string())?
            {
                return Err("discovery changed registered execution settings".into());
            }
            s.config.profiles = profiles;
            s.job = None;
            self.store(s)?;
        }
        Ok(())
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
            .map_err(|_| "startup job state poisoned")?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err("startup operation still running".into());
        }
        let result = self
            .broker
            .launch_locked(operation, &args, &self.revision(s)?, &mut slot)?;
        s.job = Some(StoredJob {
            id: result["id"].as_str().ok_or("missing job ID")?.into(),
            operation: operation.into(),
            started_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis()
                .try_into()
                .map_err(|_| "clock out of range")?,
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
        let mut saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        let mut next = saved.clone();
        next.config.name = name.into();
        self.store(&next)?;
        *saved = next;
        Ok(())
    }
    /// Switching workspaces must not orphan a planning or start job.
    pub(crate) fn ensure_idle(&self) -> Result<(), String> {
        let saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        if self.job(&saved)?["state"] == "running" || self.quota_job(&saved)?["state"] == "running"
        {
            return Err(
                "wait for the running planning or metadata job before switching workspaces".into(),
            );
        }
        drop(saved);
        self.ensure_no_live_jobs()
    }
    pub(crate) fn executable(&self) -> &Path {
        &self.broker.exe
    }
    pub(crate) fn active_run(&self) -> Result<Option<PathBuf>, String> {
        let saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        if let Some(proposal) = &saved.proposal
            && proposal.phase == "starting"
            && proposal.run.join("report.json").exists()
        {
            let report = workflow::load(&proposal.run)?;
            if report.plan.goal != saved.config.goal
                || report.repo != saved.config.repository
                || report.base != proposal.repository_base
            {
                return Err("run does not belong to the reviewed startup proposal".into());
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
            owner
                .ensure_quiescent()
                .map_err(|e| format!("retained startup job is live or unknown: {e}"))?;
        }
        Ok(())
    }
}

pub(crate) fn document() -> Result<String, String> {
    let nonce = crate::native_guard::session_id()?;
    let home = include_str!("startup.html").to_string() + include_str!("mcp.html");
    let style = format!(
        "{}\n{}\n{}",
        super::CSS,
        include_str!("startup.css"),
        include_str!("mcp.css")
    );
    let script = format!(
        "{}\nwindow.startRoom=function(){{ {} }};\n{}\n{}",
        super::I18N,
        super::JS,
        include_str!("mcp.js"),
        include_str!("startup.js")
    );
    let policy = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'nonce-{nonce}'; style-src 'nonce-{nonce}'; connect-src 'none'; img-src data:; base-uri 'none'; form-action 'none'\">"
    );
    Ok(super::language::embed(super::HTML).replace("<link rel=\"stylesheet\" href=\"/ui.css\"><script src=\"/ui.js\" defer></script>",&format!("{policy}<style nonce=\"{nonce}\">{style}</style>"))
        .replace("<body>",&format!("<body><section id=\"startup-screen\">{home}</section><section id=\"run-screen\" hidden><button id=\"startup-home\" class=\"small-button room-home\">Workspace</button>"))
        .replace("</body>",&format!("</section><script nonce=\"{nonce}\">{script}</script></body>")))
}

#[cfg(test)]
#[path = "../../tests/unit/startup.rs"]
mod tests;

mod baseline;
mod messenger;
mod request;
use request::Request;
mod dispatch;
mod planning;
mod profiles;
mod share;
mod slug;
pub(crate) use profiles::{validate_catalog, validate_name, validate_profiles};

mod mcp;
mod quota;

mod project;

mod jobs;
