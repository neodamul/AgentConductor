use crate::{
    process::{Ownership, record_owner},
    state::Store,
    workflow::{self, Report},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Seek, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

const HTML: &str = include_str!("ui/room.html");
const CSS: &str = include_str!("ui/room.css");
const JS: &str = include_str!("ui/room.js");
pub mod native;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ActionRequest {
    revision: String,
    action: Action,
}
#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Action {
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
    Resume,
    Recover,
    Stop { reason: String },
}
impl Action {
    fn label(&self) -> &'static str {
        match self {
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
            Self::Resume => "resume",
            Self::Recover => "recover",
            Self::Stop { .. } => "stop",
        }
    }
    fn args(&self, report: &Report, dir: &Path) -> Result<Vec<String>, String> {
        Ok(match self {
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
            Self::Resume if report.state == "stopped" => {
                vec!["ui-resume".into(), workflow::stop_digest(dir)?]
            }
            Self::Resume => vec!["resume-due".into()],
            Self::Recover => vec!["recover".into()],
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
}
struct Controller {
    dir: PathBuf,
    exe: PathBuf,
    token: String,
    address: SocketAddr,
    job: Mutex<Option<Job>>,
}
impl Controller {
    fn poll(&self) -> Result<serde_json::Value, String> {
        let mut slot = self.job.lock().map_err(|_| "UI job state poisoned")?;
        let Some(job) = slot.as_mut() else {
            return Ok(serde_json::Value::Null);
        };
        if job.exit.is_none()
            && let Some(status) = job.child.try_wait().map_err(|e| e.to_string())?
        {
            let code = status.code().unwrap_or(-1);
            job.exit = Some(code);
            job.owner.phase = "terminal".into();
            job.owner.exit_code = Some(code);
            record_owner(&job.logs, &job.owner)?;
            Store::open(self.dir.join("state.sqlite"))?.record(
                "ui_action_result",
                &serde_json::json!({"id":job.id,"kind":job.label,"exit_code":code}),
                &format!("UI action {} finished with exit {code}", job.label),
            )?;
        }
        Ok(
            serde_json::json!({"id":job.id,"kind":job.label,"state":match job.exit {None=>"running",Some(0)=>"completed",Some(_)=>"failed"},"exit_code":job.exit,"error":if job.exit.is_some_and(|c|c!=0){tail(&job.logs.join("stderr.log"),4096)?}else{String::new()}}),
        )
    }
    fn snapshot(&self) -> Result<serde_json::Value, String> {
        let job = self.poll()?;
        let report = workflow::load(&self.dir)?;
        let revision = workflow::revision(&report)?;
        let value = public_report(&report)?;
        let ledger = crate::gates::Ledger::open(&self.dir.join("state.sqlite"))?;
        let approvals = report
            .tasks
            .iter()
            .filter_map(|t| t.tool_request.as_deref())
            .map(|id| ledger.receipt(id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            serde_json::json!({"report":value,"revision":revision,"job":job,"tool_requests":approvals.into_iter().flatten().collect::<Vec<_>>(),"run_dir":self.dir}),
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
        if slot.as_ref().is_some_and(|j| j.exit.is_none()) {
            return Err("another UI action is still running".into());
        }
        let report = workflow::load(&self.dir)?;
        if workflow::revision(&report)? != request.revision {
            return Err("run changed since review; refresh before acting".into());
        }
        let args = request.action.args(&report, &self.dir)?;
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
            .arg("ui-action")
            .arg(&self.dir)
            .arg(&request.revision)
            .args(&args)
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
            label: request.action.label().into(),
            child,
            logs: logs.clone(),
            owner,
            exit: None,
        });
        record_owner(&logs, &slot.as_ref().unwrap().owner)?;
        Store::open(self.dir.join("state.sqlite"))?.record(
            "ui_action",
            &serde_json::json!({"id":id,"kind":request.action.label(),"revision":request.revision}),
            &format!(
                "Human UI action {} accepted; completion remains pending",
                request.action.label()
            ),
        )?;
        Ok(serde_json::json!({"accepted":true,"id":id,"kind":request.action.label()}))
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

struct Request {
    method: String,
    path: String,
    headers: std::collections::BTreeMap<String, String>,
    body: Vec<u8>,
}
fn read_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    let end = loop {
        let n = stream.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("incomplete request".into());
        }
        bytes.extend_from_slice(&buffer[..n]);
        if let Some(pos) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
            if pos > 16 * 1024 {
                return Err("request headers too large".into());
            }
            break pos + 4;
        }
        if bytes.len() > 16 * 1024 {
            return Err("request headers too large".into());
        }
    };
    let header = std::str::from_utf8(&bytes[..end]).map_err(|_| "invalid HTTP headers")?;
    let mut lines = header.split("\r\n");
    let first = lines.next().ok_or("missing request line")?;
    let parts = first.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 || parts[2] != "HTTP/1.1" || !parts[1].starts_with('/') {
        return Err("invalid request line".into());
    }
    let method = parts[0].to_owned();
    let path = parts[1].to_owned();
    let mut headers = std::collections::BTreeMap::new();
    for line in lines.filter(|s| !s.is_empty()) {
        let (key, value) = line.split_once(':').ok_or("invalid request header")?;
        if headers
            .insert(key.to_ascii_lowercase(), value.trim().to_owned())
            .is_some()
        {
            return Err("duplicate request header".into());
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err("chunked requests are unsupported".into());
    }
    let length = headers
        .get("content-length")
        .map(|s| s.parse::<usize>().map_err(|_| "invalid content length"))
        .transpose()?
        .unwrap_or(0);
    if length > 64 * 1024 {
        return Err("request body exceeds 64 KiB".into());
    }
    let mut body = bytes[end..].to_vec();
    while body.len() < length {
        let n = stream.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("incomplete request body".into());
        }
        body.extend_from_slice(&buffer[..n]);
    }
    if body.len() != length {
        return Err("pipelining is unsupported".into());
    }
    Ok(Request {
        method,
        path,
        headers,
        body,
    })
}
fn response(stream: &mut TcpStream, status: u16, kind: &str, body: &[u8], extra: &str) {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        303 => "See Other",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        _ => "Service Unavailable",
    };
    let headers = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'\r\n{extra}\r\n",
        body.len()
    );
    let _ = stream
        .write_all(headers.as_bytes())
        .and_then(|_| stream.write_all(body));
}
fn handle(mut stream: TcpStream, controller: Arc<Controller>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
    let request = match read_request(&mut stream) {
        Ok(r) => r,
        Err(e) => {
            response(&mut stream, 400, "text/plain", e.as_bytes(), "");
            return;
        }
    };
    let origin = format!("http://{}", controller.address);
    if request.headers.get("host") != Some(&controller.address.to_string())
        || request.headers.get("origin").is_some_and(|s| s != &origin)
        || request
            .headers
            .get("sec-fetch-site")
            .is_some_and(|s| s != "same-origin" && s != "none")
    {
        response(&mut stream, 403, "text/plain", b"Local origin required", "");
        return;
    }
    if request.method == "GET" && request.path == format!("/session/{}", controller.token) {
        let extra = format!(
            "Set-Cookie: ac_session_{}={}; HttpOnly; SameSite=Strict; Path=/\r\nLocation: /\r\n",
            controller.address.port(),
            controller.token
        );
        response(
            &mut stream,
            303,
            "text/plain",
            b"Local session ready",
            &extra,
        );
        return;
    }
    if !request.headers.get("cookie").is_some_and(|s| {
        s.split(';').any(|cookie| {
            cookie.trim()
                == format!(
                    "ac_session_{}={}",
                    controller.address.port(),
                    controller.token
                )
        })
    }) {
        response(
            &mut stream,
            403,
            "text/plain",
            b"Local session capability required",
            "",
        );
        return;
    }
    if request.method == "POST"
        && (request.headers.get("origin") != Some(&origin)
            || request.headers.get("content-type").map(String::as_str) != Some("application/json"))
    {
        response(
            &mut stream,
            403,
            "text/plain",
            b"Same-origin JSON action required",
            "",
        );
        return;
    }
    let outcome: Result<serde_json::Value, String> =
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/") => {
                response(
                    &mut stream,
                    200,
                    "text/html; charset=utf-8",
                    HTML.as_bytes(),
                    "",
                );
                return;
            }
            ("GET", "/ui.css") => {
                response(
                    &mut stream,
                    200,
                    "text/css; charset=utf-8",
                    CSS.as_bytes(),
                    "",
                );
                return;
            }
            ("GET", "/ui.js") => {
                response(
                    &mut stream,
                    200,
                    "text/javascript; charset=utf-8",
                    JS.as_bytes(),
                    "",
                );
                return;
            }
            ("GET", "/api/snapshot") => controller.snapshot(),
            ("GET", "/api/evidence") => controller.evidence(),
            ("GET", path) if path.starts_with("/api/task?index=") => path
                .trim_start_matches("/api/task?index=")
                .parse::<usize>()
                .map_err(|_| "invalid task index".into())
                .and_then(|i| controller.task(i)),
            ("POST", "/api/action") => serde_json::from_slice::<ActionRequest>(&request.body)
                .map_err(|e| e.to_string())
                .and_then(|r| controller.action(r)),
            _ => {
                response(&mut stream, 404, "text/plain", b"Unknown route", "");
                return;
            }
        };
    match outcome {
        Ok(value) => response(
            &mut stream,
            if request.method == "POST" { 202 } else { 200 },
            "application/json",
            value.to_string().as_bytes(),
            "",
        ),
        Err(e) => response(
            &mut stream,
            409,
            "application/json",
            serde_json::json!({"error":e}).to_string().as_bytes(),
            "",
        ),
    }
}

pub struct Server {
    controller: Arc<Controller>,
    closed: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    _window_lease: Option<crate::leases::Lease>,
}
impl Server {
    /// A native window holds one kernel lease per canonical run directory.
    pub fn start_window(dir: &Path, exe: &Path) -> Result<Self, String> {
        workflow::load(dir)?;
        let canonical = dir.canonicalize().map_err(|e| e.to_string())?;
        let lease=crate::leases::Lease::acquire(&canonical.join("ui-window.lock")).map_err(|e| if e.starts_with("lease is held:") {"Agent Room is already open for this run; close its previous window before opening another".into()} else {e})?;
        let mut server = Self::start(&canonical, exe)?;
        server._window_lease = Some(lease);
        Ok(server)
    }
    pub fn start(dir: &Path, exe: &Path) -> Result<Self, String> {
        workflow::load(dir)?;
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let token = format!(
            "{}{}",
            crate::native_guard::session_id()?,
            crate::native_guard::session_id()?
        )
        .replace('-', "");
        let controller = Arc::new(Controller {
            dir: dir.canonicalize().map_err(|e| e.to_string())?,
            exe: exe.canonicalize().map_err(|e| e.to_string())?,
            token,
            address,
            job: Mutex::new(None),
        });
        let closed = Arc::new(AtomicBool::new(false));
        let close = closed.clone();
        let engine = controller.clone();
        let active = Arc::new(AtomicUsize::new(0));
        let thread = std::thread::spawn(move || {
            for stream in listener.incoming() {
                if close.load(Ordering::Acquire) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                if active.fetch_add(1, Ordering::AcqRel) >= 8 {
                    active.fetch_sub(1, Ordering::AcqRel);
                    response(&mut stream, 503, "text/plain", b"Local UI is busy", "");
                    continue;
                }
                let engine = engine.clone();
                let active = active.clone();
                std::thread::spawn(move || {
                    handle(stream, engine);
                    active.fetch_sub(1, Ordering::AcqRel);
                });
            }
        });
        Ok(Self {
            controller,
            closed,
            thread: Some(thread),
            _window_lease: None,
        })
    }
    pub fn address(&self) -> SocketAddr {
        self.controller.address
    }
    pub fn capability(&self) -> &str {
        &self.controller.token
    }
    pub fn bootstrap_url(&self) -> String {
        format!("http://{}/session/{}", self.address(), self.capability())
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Release);
        let _ = TcpStream::connect_timeout(&self.address(), Duration::from_millis(200));
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
