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
}
#[derive(Clone, Serialize, Deserialize)]
struct Saved {
    config: Config,
    proposal: Option<Proposal>,
    job: Option<StoredJob>,
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
        });
    }
    Ok(Config {
        profiles,
        repository: PathBuf::new(),
        goal: String::new(),
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
    fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(|e| e.to_string())
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
        let current = self.broker.poll()?;
        if !current.is_null() {
            return Ok(current);
        }
        let Some(job) = &s.job else {
            return Ok(serde_json::Value::Null);
        };
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
            serde_json::json!({"id":job.id,"kind":job.operation,"state":state,"exit_code":owner.exit_code,"termination_signal":owner.termination_signal}),
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
            s.proposal = None;
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
        });
        self.store(s)?;
        Ok(result)
    }
    pub(crate) fn poll_job(&self) -> Result<(), String> {
        self.broker.poll().map(|_| ())
    }
    pub(crate) fn executable(&self) -> &Path {
        &self.broker.exe
    }
    pub(crate) fn active_run(&self) -> Result<Option<PathBuf>, String> {
        let saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        let Some(proposal) = &saved.proposal else {
            return Ok(None);
        };
        if proposal.phase != "starting" || !proposal.run.join("report.json").exists() {
            return Ok(None);
        }
        let report = workflow::load(&proposal.run)?;
        if report.plan.goal != saved.config.goal
            || report.repo != saved.config.repository
            || report.base != proposal.repository_base
        {
            return Err("run does not belong to the reviewed startup proposal".into());
        }
        Ok(Some(proposal.run.clone()))
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
    pub fn dispatch(&self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        if bytes.len() > 64 * 1024 {
            return Err("native request exceeds 64 KiB".into());
        }
        let request: Request = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        let mut saved = self.saved.lock().map_err(|_| "startup settings poisoned")?;
        let job = self.job(&saved)?;
        self.settle(&mut saved, &job)?;
        if let Request::Snapshot {} = request {
            let mut config = serde_json::to_value(&saved.config).map_err(|e| e.to_string())?;
            for p in config["profiles"]
                .as_array_mut()
                .ok_or("invalid registry")?
            {
                p.as_object_mut().unwrap().remove("environment");
            }
            let proposal = if let Some(p) = &saved.proposal {
                let path = p.directory.join("review.json");
                let mut value = serde_json::to_value(p).map_err(|e| e.to_string())?;
                if path.exists() {
                    let plan: Plan = read(&path)?;
                    let digest =
                        workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?);
                    let mut public = serde_json::to_value(plan).map_err(|e| e.to_string())?;
                    for agent in public["agents"].as_array_mut().unwrap() {
                        agent.as_object_mut().unwrap().remove("environment");
                    }
                    value["plan"] = public;
                    value["digest"] = digest.into();
                }
                value
            } else {
                serde_json::Value::Null
            };
            return Ok(
                serde_json::json!({"revision":self.revision(&saved)?,"config":config,"proposal":proposal,"job":job,"root":self.broker.dir,"templates":defaults()?.profiles,"run_available":saved.proposal.as_ref().is_some_and(|p|p.phase=="starting" && p.run.join("report.json").exists())}),
            );
        }
        if job["state"] == "running" {
            return Err("startup operation still running".into());
        }
        self.ensure_no_live_jobs()?;
        let revision = match &request {
            Request::Save { revision, .. }
            | Request::Discover { revision, .. }
            | Request::Catalog { revision, .. }
            | Request::Prepare { revision }
            | Request::Edit { revision, .. }
            | Request::Start { revision, .. } => revision,
            Request::Snapshot {} => unreachable!(),
        };
        if revision != &self.revision(&saved)? {
            return Err("startup state changed since review; refresh before acting".into());
        }
        match request {
            Request::Save { mut config, .. } => {
                validate_profiles(&config.profiles)?;
                if !config.profiles.iter().any(|p| p.id == config.coordinator) {
                    return Err("coordinator is not registered".into());
                }
                if config.goal.len() > 16 * 1024 {
                    return Err("goal exceeds 16 KiB".into());
                }
                if !config.repository.as_os_str().is_empty() {
                    config.repository = Repository::open(&config.repository)?.root;
                    if self.broker.dir.starts_with(&config.repository) {
                        return Err("startup artifacts must be outside repository".into());
                    }
                }
                for profile in &mut config.profiles {
                    if profile.environment.is_empty()
                        && let Some(old) = saved.config.profiles.iter().find(|p| p.id == profile.id)
                    {
                        profile.environment = old.environment.clone();
                    }
                }
                let next = Saved {
                    config,
                    proposal: None,
                    job: None,
                };
                self.store(&next)?;
                *saved = next;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Catalog { agent, catalog, .. } => {
                validate_catalog(&catalog)?;
                let mut next = saved.clone();
                next.config
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == agent)
                    .ok_or("unknown agent")?
                    .catalog = Some(catalog);
                next.proposal = None;
                next.job = None;
                self.store(&next)?;
                *saved = next;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Discover { agent, .. } => {
                let profile = saved
                    .config
                    .profiles
                    .iter()
                    .find(|p| p.id == agent)
                    .ok_or("unknown agent")?;
                if profile.parser == crate::protocol::Provider::Custom {
                    return Err("custom agents import an observed catalog".into());
                }
                write(
                    &self.broker.dir.join("profiles.json"),
                    &saved.config.profiles,
                )?;
                let discovery = self
                    .broker
                    .dir
                    .join(format!("models-{}", crate::native_guard::session_id()?));
                self.launch(
                    &mut saved,
                    &format!("catalog:{agent}"),
                    vec![
                        "models-discover".into(),
                        self.broker
                            .dir
                            .join("profiles.json")
                            .to_string_lossy()
                            .into(),
                        agent,
                        discovery.to_string_lossy().into(),
                    ],
                )
            }
            Request::Prepare { .. } => {
                let config = &saved.config;
                if config.goal.trim().is_empty() {
                    return Err("enter a nonempty goal".into());
                }
                validate_profiles(&config.profiles)?;
                let coordinator = config
                    .profiles
                    .iter()
                    .find(|p| p.id == config.coordinator)
                    .ok_or("coordinator not registered")?;
                crate::coordinator::read_only_args(coordinator, "{task_file}")?;
                let repo = Repository::open(&config.repository)?;
                repo.clean()?;
                if self.broker.dir.starts_with(&repo.root) {
                    return Err("startup artifacts must be outside repository".into());
                }
                write(&self.broker.dir.join("profiles.json"), &config.profiles)?;
                let directory = self
                    .broker
                    .dir
                    .join(format!("plan-{}", crate::native_guard::session_id()?));
                let run = self
                    .broker
                    .dir
                    .join(format!("run-{}", crate::native_guard::session_id()?));
                let args = vec![
                    "plan".into(),
                    self.broker
                        .dir
                        .join("profiles.json")
                        .to_string_lossy()
                        .into(),
                    config.coordinator.clone(),
                    repo.root.to_string_lossy().into(),
                    directory.to_string_lossy().into(),
                    config.goal.clone(),
                ];
                saved.proposal = Some(Proposal {
                    directory,
                    repository_base: repo.head()?,
                    run,
                    phase: "planning".into(),
                });
                self.store(&saved)?;
                self.launch(&mut saved, "plan", args)
            }
            Request::Edit {
                tasks,
                workspace_mode,
                ..
            } => {
                let p = saved.proposal.as_ref().ok_or("no generated plan")?;
                if p.phase != "ready" {
                    return Err("plan is not ready for review".into());
                }
                let plan = Plan {
                    goal: saved.config.goal.clone(),
                    agents: saved.config.profiles.clone(),
                    tasks,
                    workspace_mode,
                };
                plan.validate()?;
                write(&p.directory.join("review.json"), &plan)?;
                saved.config.workspace_mode = workspace_mode;
                self.store(&saved)?;
                Ok(serde_json::json!({"saved":true}))
            }
            Request::Start { digest, revision } => {
                let p = saved.proposal.as_ref().ok_or("no generated plan")?;
                if p.phase != "ready" {
                    return Err("plan is not ready for execution consent".into());
                }
                let path = p.directory.join("review.json");
                let plan: Plan = read(&path)?;
                plan.validate()?;
                if plan.goal != saved.config.goal
                    || serde_json::to_vec(&plan.agents).map_err(|e| e.to_string())?
                        != serde_json::to_vec(&saved.config.profiles).map_err(|e| e.to_string())?
                {
                    return Err("reviewed plan changed the registered profiles or goal".into());
                }
                if workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?)
                    != digest
                {
                    return Err("approve the exact reviewed plan digest".into());
                }
                workflow::validate_permissions(&plan)?;
                let repo = Repository::open(&saved.config.repository)?;
                repo.clean()?;
                if repo.head()? != p.repository_base {
                    return Err("repository changed since planning; generate a new plan".into());
                }
                crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record("plan_execution_consent",&serde_json::json!({"digest":digest,"revision":revision,"repository_base":p.repository_base,"plan":path}),"Human approved the exact reviewed startup plan")?;
                let run = p.run.clone();
                let args = vec![
                    "startup-run".into(),
                    path.to_string_lossy().into(),
                    repo.root.to_string_lossy().into(),
                    run.to_string_lossy().into(),
                    digest,
                    p.repository_base.clone(),
                ];
                saved.proposal.as_mut().unwrap().phase = "starting".into();
                self.store(&saved)?;
                let mut result = self.launch(&mut saved, "start", args)?;
                result["run"] = serde_json::json!(run);
                Ok(result)
            }
            Request::Snapshot {} => unreachable!(),
        }
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
mod tests {
    #[test]
    fn oversized_settings_leave_no_pending_file_or_partial_update() {
        let root = std::env::temp_dir().join(format!(
            "ac-startup-write-{}",
            crate::native_guard::session_id().unwrap()
        ));
        std::fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let path = root.join("startup.json");
        super::write(&path, &"original").unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(
            super::write(&path, &"x".repeat(65536))
                .unwrap_err()
                .contains("exceeds 64 KiB")
        );
        assert_eq!(std::fs::read(&path).unwrap(), original);
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
