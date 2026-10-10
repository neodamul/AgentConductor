use crate::{
    process::{Ownership, record_owner},
    state::Store,
    workflow::{self, Report},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Seek},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

const HTML: &str = include_str!("ui/room.html");
const CSS: &str = include_str!("ui/room.css");
const JS: &str = include_str!("ui/room.js");
pub mod native;
pub mod setup;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ActionRequest {
    revision: String,
    action: Action,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
    Quota { agent: String },
    Approve { sha: String },
    Merge { sha: String },
    Reject { reason: String },
    Revise { task: String, feedback: String },
    Reassign { task: String, agent: String },
    Model { task: String, model: String },
    Schedule { task: String, at: String },
    ToolApprove { sha: String },
    ToolReject { sha: String },
    ToolRefresh { sha: String },
    Resume {},
    Recover {},
    Stop { reason: String },
}
impl Action {
    fn label(&self) -> &'static str {
        match self {
            Self::Quota { .. } => "quota",
            Self::Approve { .. } => "approve",
            Self::Merge { .. } => "merge",
            Self::Reject { .. } => "reject",
            Self::Revise { .. } => "revise",
            Self::Reassign { .. } => "reassign",
            Self::Model { .. } => "model",
            Self::Schedule { .. } => "schedule",
            Self::ToolApprove { .. } => "tool_approve",
            Self::ToolReject { .. } => "tool_reject",
            Self::ToolRefresh { .. } => "tool_refresh",
            Self::Resume {} => "resume",
            Self::Recover {} => "recover",
            Self::Stop { .. } => "stop",
        }
    }
    fn args(&self, report: &Report, dir: &Path) -> Result<Vec<String>, String> {
        Ok(match self {
            Self::Quota { agent } => {
                if !report
                    .plan
                    .agents
                    .iter()
                    .any(|profile| &profile.id == agent)
                {
                    return Err("unknown quota agent".into());
                }
                vec!["quota-refresh".into(), agent.clone()]
            }
            Self::Approve { sha } => vec!["approve".into(), sha.clone()],
            Self::Merge { sha } => {
                if report.state != "approved"
                    || report.candidate.as_ref().is_none_or(|c| &c.commit != sha)
                {
                    return Err("merge requires the exact approved candidate".into());
                }
                vec!["merge".into()]
            }
            Self::Reject { reason } => vec!["reject".into(), reason.clone()],
            Self::Revise { task, feedback } => {
                vec!["revise".into(), task.clone(), feedback.clone()]
            }
            Self::Reassign { task, agent } => vec!["reassign".into(), task.clone(), agent.clone()],
            Self::Model { task, model } => {
                vec!["session-model".into(), task.clone(), model.clone()]
            }
            Self::Schedule { task, at } => vec!["schedule".into(), task.clone(), at.clone()],
            Self::ToolApprove { sha } => vec!["tool-approve".into(), sha.clone()],
            Self::ToolReject { sha } => vec!["tool-reject".into(), sha.clone()],
            Self::ToolRefresh { sha } => vec!["tool-refresh".into(), sha.clone()],
            Self::Resume {} if report.state == "stopped" => {
                vec!["ui-resume".into(), workflow::stop_digest(dir)?]
            }
            Self::Resume {} => vec!["resume-due".into()],
            Self::Recover {} => vec!["recover".into()],
            Self::Stop { .. } => return Err("stop is handled directly".into()),
        })
    }
}

struct Job {
    id: String,
    label: String,
    child: Child,
    logs: PathBuf,
    owner: Ownership,
    exit: Option<i32>,
    finished: bool,
}
struct Controller {
    dir: PathBuf,
    exe: PathBuf,
    job: Mutex<Option<Job>>,
}
impl Controller {
    fn poll(&self) -> Result<serde_json::Value, String> {
        let mut slot = self.job.lock().map_err(|_| "UI job state poisoned")?;
        let Some(job) = slot.as_mut() else {
            return Ok(serde_json::Value::Null);
        };
        if !job.finished
            && let Some(status) = job.child.try_wait().map_err(|e| e.to_string())?
        {
            let code = status.code();
            job.exit = code;
            job.finished = true;
            job.owner.phase = "terminal".into();
            job.owner.exit_code = code;
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                job.owner.termination_signal = status.signal();
            }
            record_owner(&job.logs, &job.owner)?;
            Store::open(self.dir.join("state.sqlite"))?.record(
                "ui_action_result",
                &serde_json::json!({"id":job.id,"kind":job.label,"exit_code":code,"termination_signal":job.owner.termination_signal}),
                &format!("UI action {} finished with exit {code:?}, signal {:?}", job.label, job.owner.termination_signal),
            )?;
        }
        Ok(
            serde_json::json!({"id":job.id,"kind":job.label,"state":if !job.finished {"running"} else if job.exit==Some(0) {"completed"} else {"failed"},"exit_code":job.exit,"termination_signal":job.owner.termination_signal,"error":if job.finished && job.exit!=Some(0){tail(&job.logs.join("stderr.log"),4096)?}else{String::new()}}),
        )
    }
    fn snapshot(&self) -> Result<serde_json::Value, String> {
        let job = self.poll()?;
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let value = public_report(&report)?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let quotas = report
            .plan
            .agents
            .iter()
            .map(|profile| {
                Ok((
                    profile.id.clone(),
                    crate::quota::stored(&store, profile, now)?,
                ))
            })
            .collect::<Result<serde_json::Map<String, serde_json::Value>, String>>()?;
        let ledger = crate::gates::Ledger::open(&self.dir.join("state.sqlite"))?;
        let approvals = report
            .tasks
            .iter()
            .filter_map(|t| t.tool_request.as_deref())
            .map(|id| ledger.receipt(id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            serde_json::json!({"report":value,"revision":revision,"quotas":quotas,"job":job,"tool_requests":approvals.into_iter().flatten().collect::<Vec<_>>(),"run_dir":self.dir,"icons":crate::icons::icon_map(report.plan.agents.iter().map(|a|a.parser))}),
        )
    }
    fn action(&self, request: ActionRequest) -> Result<serde_json::Value, String> {
        if let Action::Stop { reason } = &request.action {
            workflow::stop(&self.dir, reason)?;
            return Ok(
                serde_json::json!({"accepted":true,"kind":"stop","message":"Stop requested; inspect task status until processes are stopped."}),
            );
        }
        self.poll()?;
        let mut slot = self.job.lock().map_err(|_| "UI job state poisoned")?;
        if slot.as_ref().is_some_and(|j| !j.finished) {
            return Err("another UI action is still running".into());
        }
        let report = workflow::load(&self.dir)?;
        if workflow::revision(&report)? != request.revision {
            return Err("run changed since review; refresh before acting".into());
        }
        let args = request.action.args(&report, &self.dir)?;
        let mut command = vec![
            "ui-action".into(),
            self.dir.to_string_lossy().into_owned(),
            request.revision.clone(),
        ];
        command.extend(args);
        self.launch_locked(
            request.action.label(),
            &command,
            &request.revision,
            &mut slot,
        )
    }
    fn launch_locked(
        &self,
        label: &str,
        args: &[String],
        revision: &str,
        slot: &mut Option<Job>,
    ) -> Result<serde_json::Value, String> {
        let id = crate::native_guard::session_id()?;
        let root = self.dir.join(format!("coordinator-ui-{id}"));
        let logs = root.join("process");
        fs::create_dir(&root)
            .and_then(|_| fs::create_dir(&logs))
            .map_err(|e| e.to_string())?;
        let mut owner = Ownership {
            coordinator_pid: std::process::id(),
            pid: None,
            process_group: None,
            phase: "not_spawned".into(),
            exit_code: None,
            termination_signal: None,
            stopped: false,
            timed_out: false,
        };
        record_owner(&logs, &owner)?;
        let out = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(logs.join("stdout.log"))
            .map_err(|e| e.to_string())?;
        let err = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(logs.join("stderr.log"))
            .map_err(|e| e.to_string())?;
        let mut command = Command::new(&self.exe);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(out)
            .stderr(err);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        owner.phase = "launching".into();
        record_owner(&logs, &owner)?;
        let child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                owner.phase = "not_spawned".into();
                record_owner(&logs, &owner)?;
                return Err(e.to_string());
            }
        };
        owner.pid = Some(child.id());
        owner.process_group = Some(child.id() as i32);
        owner.phase = "running".into();
        *slot = Some(Job {
            id: id.clone(),
            label: label.into(),
            child,
            logs: logs.clone(),
            owner,
            exit: None,
            finished: false,
        });
        record_owner(&logs, &slot.as_ref().unwrap().owner)?;
        Store::open(self.dir.join("state.sqlite"))?.record(
            "ui_action",
            &serde_json::json!({"id":id,"kind":label,"revision":revision}),
            &format!(
                "Human UI action {} accepted; completion remains pending",
                label
            ),
        )?;
        Ok(serde_json::json!({"accepted":true,"id":id,"kind":label}))
    }
    fn evidence(&self) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let path = self.dir.join("diff.patch");
        let (diff, truncated) = if path.exists() {
            bounded(&path, 512 * 1024)?
        } else {
            (String::new(), false)
        };
        if !truncated
            && let Some(expected) = &report.diff_hash
            && workflow::digest(diff.as_bytes()) != *expected
        {
            return Err("candidate diff changed; verification required".into());
        }
        let tests=report.tests.iter().map(|t|Ok(serde_json::json!({"task":t.task,"exit_code":t.exit_code,"timed_out":t.timed_out,"stdout":tail(&t.stdout,8192)?,"stderr":tail(&t.stderr,8192)?}))).collect::<Result<Vec<_>,String>>()?;
        if workflow::revision(&workflow::load(&self.dir)?)? != revision {
            return Err("run changed while collecting review evidence".into());
        }
        Ok(
            serde_json::json!({"revision":revision,"report":public_report(&report)?,"diff":diff,"truncated":truncated,"test_output":tests}),
        )
    }
    fn task(&self, index: usize) -> Result<serde_json::Value, String> {
        let report = workflow::load(&self.dir)?;
        let task = report.tasks.get(index).ok_or("unknown task index")?;
        let store = Store::open(self.dir.join("state.sqlite"))?;
        let events = store.task_events(&task.id, task.attempt as u64)?;
        let mut usage = serde_json::Value::Null;
        let events=events.into_iter().rev().take(100).map(|e|{
            if usage.is_null() {usage=e.pointer("/payload/usage").or_else(||e.pointer("/payload/message/usage")).cloned().unwrap_or(serde_json::Value::Null);}
            let payload=e["payload"].to_string();serde_json::json!({"seq":e["seq"],"kind":e["kind"],"session_id":e["session_id"],"raw_offset":e["raw_offset"],"text":payload.chars().take(1600).collect::<String>()})
        }).collect::<Vec<_>>();
        let logs = task.artifacts.join("process");
        Ok(
            serde_json::json!({"task":task,"events":events.into_iter().rev().collect::<Vec<_>>(),"usage":usage,"stdout":if logs.join("stdout.jsonl").exists(){tail(&logs.join("stdout.jsonl"),64*1024)?}else{String::new()},"stderr":if logs.join("stderr.log").exists(){tail(&logs.join("stderr.log"),32*1024)?}else{String::new()},"knowledge":crate::knowledge::status(&self.dir,&task.cwd)?}),
        )
    }
}

fn public_report(report: &Report) -> Result<serde_json::Value, String> {
    let mut value = serde_json::to_value(report).map_err(|e| e.to_string())?;
    if let Some(agents) = value
        .pointer_mut("/plan/agents")
        .and_then(serde_json::Value::as_array_mut)
    {
        for agent in agents {
            agent
                .as_object_mut()
                .ok_or("invalid profile")?
                .remove("environment");
        }
    }
    Ok(value)
}

// Walk directory descriptors rather than checking paths and reopening them.
// Each component remains bound even if an agent renames an ancestor concurrently.
#[cfg(unix)]
fn open_artifact(path: &Path) -> Result<fs::File, String> {
    use std::{
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{ffi::OsStrExt, fs::MetadataExt},
        },
        path::Component,
    };
    if !path.is_absolute() {
        return Err("artifact path must be absolute".into());
    }
    let mut parent = fs::File::open("/").map_err(|e| e.to_string())?;
    let mut components = path
        .components()
        .filter(|c| !matches!(c, Component::RootDir))
        .peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err("invalid artifact path component".into());
        };
        let name = std::ffi::CString::new(name.as_bytes()).map_err(|e| e.to_string())?;
        let final_component = components.peek().is_none();
        let flags = libc::O_RDONLY
            | libc::O_CLOEXEC
            | libc::O_NOFOLLOW
            | libc::O_NONBLOCK
            | if final_component {
                0
            } else {
                libc::O_DIRECTORY
            };
        let descriptor = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
        if descriptor < 0 {
            return Err(format!(
                "artifact open refused: {}",
                std::io::Error::last_os_error()
            ));
        }
        parent = unsafe { fs::File::from_raw_fd(descriptor) };
        if final_component {
            let metadata = parent.metadata().map_err(|e| e.to_string())?;
            if !metadata.is_file() || metadata.nlink() != 1 {
                return Err("artifact must be a single-link regular file".into());
            }
            return Ok(parent);
        }
    }
    Err("artifact must be a regular file".into())
}
#[cfg(not(unix))]
fn open_artifact(path: &Path) -> Result<fs::File, String> {
    if fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("artifact alias refused".into());
    }
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("artifact must be regular".into());
    }
    Ok(file)
}

fn bounded(path: &Path, limit: usize) -> Result<(String, bool), String> {
    let mut bytes = Vec::new();
    open_artifact(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let truncated = bytes.len() > limit;
    bytes.truncate(limit);
    Ok((String::from_utf8_lossy(&bytes).into(), truncated))
}
fn tail(path: &Path, limit: usize) -> Result<String, String> {
    let mut file = open_artifact(path)?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    file.seek(std::io::SeekFrom::Start(len.saturating_sub(limit as u64)))
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(limit as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&bytes).into())
}

/// Only explicit controller operations can cross the native boundary.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Briefing {},
    Snapshot {},
    Evidence {},
    Task { index: usize },
    Action { revision: String, action: Action },
}

pub struct Room {
    controller: Mutex<Option<Arc<Controller>>>,
    startup: Option<setup::Startup>,
    run_lease: Mutex<Option<crate::leases::Lease>>,
    _window_lease: crate::leases::Lease,
}
impl Room {
    /// A native window holds one kernel lease per canonical run directory.
    pub fn open(dir: &Path, exe: &Path) -> Result<Self, String> {
        workflow::load(dir)?;
        let dir = dir.canonicalize().map_err(|e| e.to_string())?;
        let lease = crate::leases::Lease::acquire(&dir.join("ui-window.lock")).map_err(|e|
            if e.starts_with("lease is held:") {
                "Agent Room is already open for this run; close its previous window before opening another".into()
            } else { e })?;
        Ok(Self {
            controller: Mutex::new(Some(Arc::new(Controller {
                dir,
                exe: exe.canonicalize().map_err(|e| e.to_string())?,
                job: Mutex::new(None),
            }))),
            startup: None,
            run_lease: Mutex::new(None),
            _window_lease: lease,
        })
    }
    pub fn startup(root: &Path, exe: &Path) -> Result<Self, String> {
        let startup = setup::Startup::open(root, exe)?;
        let lease = crate::leases::Lease::acquire(&root.join("ui-window.lock"))?;
        Ok(Self {
            controller: Mutex::new(None),
            startup: Some(startup),
            run_lease: Mutex::new(None),
            _window_lease: lease,
        })
    }
    pub fn dispatch(&self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        if bytes.len() > 64 * 1024 {
            return Err("native request exceeds 64 KiB".into());
        }
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Navigation {
            OpenRun {},
            Home {},
        }
        let kind: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if matches!(kind["kind"].as_str(), Some("open_run" | "home")) {
            let startup = self.startup.as_ref().ok_or("startup surface unavailable")?;
            match serde_json::from_slice::<Navigation>(bytes).map_err(|e| e.to_string())? {
                Navigation::OpenRun {} => {
                    let mut controller = self
                        .controller
                        .lock()
                        .map_err(|_| "surface state poisoned")?;
                    if controller.is_none() {
                        let dir = startup.active_run()?.ok_or("run is not ready to open")?;
                        let lease = crate::leases::Lease::acquire(&dir.join("ui-window.lock"))?;
                        *controller = Some(Arc::new(Controller {
                            dir,
                            exe: startup.executable().to_owned(),
                            job: Mutex::new(None),
                        }));
                        *self
                            .run_lease
                            .lock()
                            .map_err(|_| "window lease state poisoned")? = Some(lease);
                    }
                }
                Navigation::Home {} => {
                    let mut controller = self
                        .controller
                        .lock()
                        .map_err(|_| "surface state poisoned")?;
                    if let Some(active) = controller.as_ref()
                        && active.poll()?["state"] == "running"
                    {
                        return Err(
                            "wait for the accepted Room action before returning home".into()
                        );
                    }
                    *controller = None;
                    *self
                        .run_lease
                        .lock()
                        .map_err(|_| "window lease state poisoned")? = None;
                }
            }
            return Ok(serde_json::json!({"opened":true}));
        }
        let controller = self
            .controller
            .lock()
            .map_err(|_| "surface state poisoned")?
            .clone();
        let Some(controller) = controller else {
            return self
                .startup
                .as_ref()
                .ok_or("no active surface")?
                .dispatch(bytes);
        };
        let request = serde_json::from_slice::<Request>(bytes).map_err(|e| e.to_string())?;
        if let Request::Briefing {} = &request {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis();
            return crate::briefing::collect(
                &controller.dir,
                u64::try_from(now).map_err(|_| "briefing clock overflow")?,
            );
        }
        if let Some(startup) = &self.startup {
            startup.poll_job()?;
        }
        match request {
            Request::Briefing {} => unreachable!("briefing returned before polling mutations"),
            Request::Snapshot {} => controller.snapshot(),
            Request::Evidence {} => controller.evidence(),
            Request::Task { index } => controller.task(index),
            Request::Action { revision, action } => {
                controller.action(ActionRequest { revision, action })
            }
        }
    }
    pub fn document(&self) -> Result<String, String> {
        if self.startup.is_some() {
            return setup::document();
        }
        let nonce = crate::native_guard::session_id()?;
        let policy = format!(
            "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; script-src 'nonce-{nonce}'; style-src 'nonce-{nonce}'; connect-src 'none'; img-src data:; base-uri 'none'; form-action 'none'\">"
        );
        Ok(HTML
            .replace(
                "<link rel=\"stylesheet\" href=\"/ui.css\"><script src=\"/ui.js\" defer></script>",
                &format!("{policy}<style nonce=\"{nonce}\">{CSS}</style>"),
            )
            .replace(
                "</body>",
                &format!("<script nonce=\"{nonce}\">{JS}</script></body>"),
            ))
    }
}
