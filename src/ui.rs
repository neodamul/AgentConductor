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
pub mod picker;
pub mod setup;
pub mod workspaces;

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
    startup: Mutex<Option<Arc<setup::Startup>>>,
    run_lease: Mutex<Option<crate::leases::Lease>>,
    window_lease: Mutex<crate::leases::Lease>,
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
            startup: Mutex::new(None),
            run_lease: Mutex::new(None),
            window_lease: Mutex::new(lease),
        })
    }
    pub fn startup(root: &Path, exe: &Path) -> Result<Self, String> {
        let startup = setup::Startup::open(root, exe)?;
        let lease = crate::leases::Lease::acquire(&root.join("ui-window.lock"))?;
        Ok(Self {
            controller: Mutex::new(None),
            startup: Mutex::new(Some(Arc::new(startup))),
            run_lease: Mutex::new(None),
            window_lease: Mutex::new(lease),
        })
    }
    fn surface(&self) -> Result<Option<Arc<setup::Startup>>, String> {
        Ok(self
            .startup
            .lock()
            .map_err(|_| "surface state poisoned")?
            .clone())
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
            PickFolder {},
            PickFile {},
            Workspaces {},
            SwitchWorkspace { path: PathBuf },
            CreateWorkspace { name: String },
            ForgetWorkspace { path: PathBuf },
            SavedWorkspaces {},
            OpenWorkspace { path: PathBuf },
        }
        let kind: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        if matches!(
            kind["kind"].as_str(),
            Some(
                "open_run"
                    | "home"
                    | "pick_folder"
                    | "pick_file"
                    | "workspaces"
                    | "switch_workspace"
                    | "create_workspace"
                    | "forget_workspace"
                    | "saved_workspaces"
                    | "open_workspace"
            )
        ) {
            let startup = self.surface()?.ok_or("startup surface unavailable")?;
            match serde_json::from_slice::<Navigation>(bytes).map_err(|e| e.to_string())? {
                Navigation::PickFolder {} => {
                    return Ok(serde_json::json!({"path":picker::pick_folder()?}));
                }
                Navigation::PickFile {} => {
                    return Ok(serde_json::json!({"path":picker::pick_file()?}));
                }
                Navigation::Workspaces {} => return self.workspaces(),
                Navigation::SwitchWorkspace { path } => return self.switch_workspace(&path),
                Navigation::CreateWorkspace { name } => return self.create_workspace(&name),
                Navigation::ForgetWorkspace { path } => return self.forget_workspace(&path),
                Navigation::SavedWorkspaces {} => return self.saved_workspaces(),
                Navigation::OpenWorkspace { path } => return self.open_workspace(&path),
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
            return self.surface()?.ok_or("no active surface")?.dispatch(bytes);
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
        if let Some(startup) = self.surface()? {
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
        if self.surface()?.is_some() {
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

mod controller;
use controller::*;
