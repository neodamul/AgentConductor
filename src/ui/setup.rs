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
}
#[derive(Clone, Serialize, Deserialize)]
struct Proposal {
    directory: PathBuf,
    repository_base: String,
    run: PathBuf,
    phase: String,
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
    config: Config,
    proposal: Option<Proposal>,
    job: Option<StoredJob>,
    /// Most recently started run, kept when the editable proposal changes (DD-39).
    #[serde(default)]
    last_run: Option<PathBuf>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Snapshot {},
    Save {
        revision: String,
        config: Config,
    },
    Discover {
        revision: String,
        agent: String,
    },
    Catalog {
        revision: String,
        agent: String,
        catalog: ModelCatalog,
    },
    Prepare {
        revision: String,
    },
    Draft {
        revision: String,
    },
    BaselineStatus {},
    CommitBaseline {
        revision: String,
    },
    IgnoreOsFiles {
        revision: String,
    },
    SaveProfiles {
        revision: String,
        profiles: Vec<AgentProfile>,
    },
    Edit {
        revision: String,
        tasks: Vec<Task>,
        workspace_mode: WorkspaceMode,
    },
    Start {
        revision: String,
        digest: String,
    },
}

/// Local startup state. Only explicit native requests launch a trusted CLI child.
pub struct Startup {
    broker: Controller,
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
            executable: invocation.executable,
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
            quota_group: None,
            fallback: None,
            cooperative: Default::default(),
        });
    }
    Ok(Config {
        profiles,
        repository: PathBuf::new(),
        goal: String::new(),
        name: String::new(),
        roles: Default::default(),
        coordinator: "codex".into(),
        workspace_mode: WorkspaceMode::Shared,
    })
}
fn validate_profiles(profiles: &[AgentProfile]) -> Result<(), String> {
    if profiles.is_empty() || profiles.len() > 16 {
        return Err("register 1..16 CLI profiles".into());
    }
    let probe = Plan {
        goal: "validate registration".into(),
        workspace_mode: WorkspaceMode::Shared,
        agents: profiles.to_vec(),
        tasks: (0..3)
            .map(|i| Task {
                id: format!("registration-{i}"),
                agent: profiles[0].id.clone(),
                prompt: "validation only".into(),
                paths: vec![format!("registration-{i}")],
                depends_on: vec![],
                test_command: vec!["true".into()],
                model: None,
                role: None,
            })
            .collect(),
    };
    probe.validate()?;
    for p in profiles {
        if p.executable.contains('\0')
            || p.args.iter().any(|a| {
                a.contains('\0')
                    || a.starts_with("--dangerously")
                    || matches!(a.as_str(), "--yolo" | "--auto")
            })
        {
            return Err("invalid CLI argv or permission bypass".into());
        }
        if !matches!(
            p.permission_mode.as_str(),
            "unverified" | "native_sandbox" | "native_callback" | "read_only" | "external_sandbox"
        ) {
            return Err("unknown permission capability".into());
        }
        if let Some(catalog) = &p.catalog {
            validate_catalog(catalog)?;
        }
    }
    Ok(())
}
fn validate_catalog(c: &ModelCatalog) -> Result<(), String> {
    if c.models.is_empty()
        || c.models.len() > 1024
        || c.queried_at <= 0
        || c.source.trim().is_empty()
        || c.cli_version.trim().is_empty()
        || c.models
            .iter()
            .any(|m| m.is_empty() || m.starts_with('-') || m.chars().any(char::is_whitespace))
    {
        return Err("catalog requires observed model IDs and query provenance".into());
    }
    Ok(())
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
                config: defaults()?,
                proposal: None,
                job: None,
                last_run: None,
            };
            write(&settings, &s)?;
            s
        };
        Ok(Self {
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
        let mut current = self.broker.poll()?;
        let Some(job) = &s.job else {
            return Ok(current);
        };
        let log = self
            .broker
            .dir
            .join(format!("coordinator-ui-{}/process", job.id));
        if !current.is_null() {
            if current["id"] == job.id.as_str() {
                current["started_ms"] = job.started_ms.into();
                current["log"] = serde_json::json!(log);
            }
            return Ok(current);
        }
        let owner: crate::process::Ownership = read(
            &self
                .broker
                .dir
                .join(format!("coordinator-ui-{}/process/owner.json", job.id)),
        )?;
        let state = if owner.phase == "terminal" {
            if owner.exit_code == Some(0) {
                "completed"
            } else {
                "failed"
            }
        } else if owner.ensure_quiescent().is_ok() {
            "interrupted"
        } else {
            "running"
        };
        Ok(
            serde_json::json!({"id":job.id,"kind":job.operation,"state":state,"exit_code":owner.exit_code,"termination_signal":owner.termination_signal,"started_ms":job.started_ms,"log":log}),
        )
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
        self.broker.poll().map(|_| ())
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
        if self.job(&saved)?["state"] == "running" {
            return Err("wait for the running planning job before switching workspaces".into());
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
    let home = include_str!("startup.html");
    let style = format!("{}\n{}", super::CSS, include_str!("startup.css"));
    let script = format!(
        "window.startRoom=function(){{ {} }};\n{}",
        super::JS,
        include_str!("startup.js")
    );
    let policy = format!(
        "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'nonce-{nonce}'; style-src 'nonce-{nonce}'; connect-src 'none'; img-src data:; base-uri 'none'; form-action 'none'\">"
    );
    Ok(super::HTML.replace("<link rel=\"stylesheet\" href=\"/ui.css\"><script src=\"/ui.js\" defer></script>",&format!("{policy}<style nonce=\"{nonce}\">{style}</style>"))
        .replace("<body>",&format!("<body><section id=\"startup-screen\">{home}</section><section id=\"run-screen\" hidden><button id=\"startup-home\" class=\"small-button room-home\">Workspace</button>"))
        .replace("</body>",&format!("</section><script nonce=\"{nonce}\">{script}</script></body>")))
}

#[cfg(test)]
#[path = "../../tests/unit/startup.rs"]
mod tests;

mod baseline;
mod dispatch;
mod profiles;
pub(crate) use profiles::validate_name;
