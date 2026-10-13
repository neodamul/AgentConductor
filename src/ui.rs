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
    process::{Child, Stdio},
    sync::{Arc, Mutex},
};

const HTML: &str = include_str!("ui/room.html");
const CSS: &str = include_str!("ui/room.css");
const JS: &str = include_str!("ui/room.js");
/// ID catalogs and translation runtime, loaded before either page script.
const I18N: &str = concat!(
    "const AC_MESSAGES={en:",
    include_str!("ui/messages/en.json"),
    ",ko:",
    include_str!("ui/messages/ko.json"),
    "};\n",
    include_str!("ui/i18n.js")
);
mod cache;
mod diff;
pub mod language;
pub mod messages;
pub mod native;
pub mod picker;
pub mod setup;
pub mod workspaces;

fn public_report(report: &Report) -> Result<serde_json::Value, String> {
    let mut value = serde_json::to_value(report).map_err(|e| e.to_string())?;
    value["direct_workspace"] = serde_json::json!(
        !report.repo.join(".git").exists()
            && crate::folder::external_metadata(&report.repo)?.is_some()
    );
    if let Some(agents) = value
        .pointer_mut("/plan/agents")
        .and_then(serde_json::Value::as_array_mut)
    {
        for agent in agents {
            agent
                .as_object_mut()
                .ok_or(crate::ui::messages::text("native.invalid_profile"))?
                .remove("environment");
        }
    }
    if let Some(folder) = value
        .get_mut("folder")
        .and_then(serde_json::Value::as_object_mut)
    {
        folder.remove("files");
    }
    // DD-113: why the orchestrator's integration verification failed, readable in the room.
    if let Some(tests) = value.get_mut("tests").and_then(|t| t.as_array_mut()) {
        for (test, source) in tests.iter_mut().zip(&report.tests) {
            if source.exit_code != Some(0) || source.timed_out {
                let output = [&source.stderr, &source.stdout]
                    .iter()
                    .filter_map(|path| tail(path, 2048).ok())
                    .find(|text| !text.trim().is_empty())
                    .unwrap_or_default();
                test["output"] = serde_json::json!(output);
            }
        }
    }
    Ok(value)
}

// Walk directory descriptors rather than checking paths and reopening them.
// Each component remains bound even if an agent renames an ancestor concurrently.
#[cfg(unix)]
pub(crate) fn open_artifact(path: &Path) -> Result<fs::File, String> {
    use std::{
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::{ffi::OsStrExt, fs::MetadataExt},
        },
        path::Component,
    };
    if !path.is_absolute() {
        return Err(crate::ui::messages::text("native.artifact_path_must_be_absolute").into());
    }
    let mut parent = fs::File::open("/").map_err(|e| e.to_string())?;
    let mut components = path
        .components()
        .filter(|c| !matches!(c, Component::RootDir))
        .peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(crate::ui::messages::text("native.invalid_artifact_path_component").into());
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
            return Err(crate::ui::messages::render(
                "native.artifact_open_refused",
                &[("value1", &std::io::Error::last_os_error().to_string())],
            ));
        }
        parent = unsafe { fs::File::from_raw_fd(descriptor) };
        if final_component {
            let metadata = parent.metadata().map_err(|e| e.to_string())?;
            if !metadata.is_file() || metadata.nlink() != 1 {
                return Err(crate::ui::messages::text(
                    "native.artifact_must_be_a_single_link_regular_file",
                )
                .into());
            }
            return Ok(parent);
        }
    }
    Err(crate::ui::messages::text("native.artifact_must_be_a_regular_file").into())
}
#[cfg(not(unix))]
pub(crate) fn open_artifact(path: &Path) -> Result<fs::File, String> {
    if fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err(crate::ui::messages::text("native.artifact_alias_refused").into());
    }
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err(crate::ui::messages::text("native.artifact_must_be_regular").into());
    }
    Ok(file)
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
/// The value with every string longer than `limit` characters shortened.
fn clip_strings(value: &serde_json::Value, limit: usize) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::String(s) if s.chars().count() > limit => {
            Value::String(s.chars().take(limit).collect::<String>() + "…")
        }
        Value::Array(items) => items.iter().map(|v| clip_strings(v, limit)).collect(),
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| (k.clone(), clip_strings(v, limit)))
            .collect(),
        other => other.clone(),
    }
}

/// Only explicit controller operations can cross the native boundary.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Briefing {},
    Snapshot {
        /// DD-71: icon data URLs only on request.
        #[serde(default)]
        icons: bool,
    },
    Evidence {},
    Task {
        index: usize,
    },
    /// DD-117: the coordinator's own work history, shown like a task's activity.
    Coordinator {},
    Action {
        revision: String,
        action: Action,
    },
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
        // A completed phase is recorded even if its goal folder moved after the run.
        let _ = workflow::sync_phase(&dir);
        let lease = crate::leases::Lease::acquire(&dir.join("ui-window.lock")).map_err(|e| {
            if e.starts_with("lease is held:") {
                crate::ui::messages::text("native.agent_room_is_already_open_for_this_run").into()
            } else {
                e
            }
        })?;
        let enabled = workflow::auto_resume_enabled(&dir);
        let controller = Arc::new(Controller {
            dir,
            exe: exe.canonicalize().map_err(|e| e.to_string())?,
            job: Mutex::new(None),
        });
        controller.automatic(enabled)?;
        Ok(Self {
            controller: Mutex::new(Some(controller)),
            startup: Mutex::new(None),
            run_lease: Mutex::new(None),
            window_lease: Mutex::new(lease),
        })
    }
    pub fn startup(root: &Path, exe: &Path) -> Result<Self, String> {
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let _navigation = crate::leases::Lease::workspace(root)?;
        let lease = crate::leases::Lease::acquire(&root.join("ui-window.lock"))?;
        let startup = setup::Startup::open(root, exe)?;
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
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?
            .clone())
    }
    pub fn dispatch(&self, bytes: &[u8]) -> Result<serde_json::Value, String> {
        if bytes.len() > 64 * 1024 {
            return Err(crate::ui::messages::text("native.native_request_exceeds_64_kib").into());
        }
        use crate::cooperative::route::Preference;
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Navigation {
            OpenRun {},
            Home {},
            PickFolder {},
            PickFile {},
            PickExecutable {},
            ResolveExecutable {
                executable: String,
            },
            Workspaces {},
            SwitchWorkspace {
                path: PathBuf,
            },
            CreateWorkspace {
                name: String,
            },
            ForgetWorkspace {
                path: PathBuf,
            },
            DeleteWorkspace {
                path: PathBuf,
                name: String,
                replacement: Option<String>,
            },
            SavedWorkspaces {},
            OpenWorkspace {
                path: PathBuf,
            },
            SetLanguage {
                language: String,
            },
            SetTheme {
                theme: String,
            },
            SignalTransport {},
            SetSignalTransport {
                preference: Preference,
            },
        }
        let kind: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
        // DD-135: update checks and installs work from any surface.
        if let Some(update @ ("update_status" | "check_update" | "apply_update")) =
            kind["kind"].as_str()
        {
            return self.update(update);
        }
        if matches!(
            kind["kind"].as_str(),
            Some(
                "open_run"
                    | "home"
                    | "pick_folder"
                    | "pick_file"
                    | "pick_executable"
                    | "resolve_executable"
                    | "workspaces"
                    | "switch_workspace"
                    | "create_workspace"
                    | "forget_workspace"
                    | "delete_workspace"
                    | "saved_workspaces"
                    | "open_workspace"
                    | "set_language"
                    | "set_theme"
                    | "signal_transport"
                    | "set_signal_transport"
            )
        ) {
            let startup = self.surface()?.ok_or(crate::ui::messages::text(
                "native.startup_surface_unavailable",
            ))?;
            match serde_json::from_slice::<Navigation>(bytes).map_err(|e| e.to_string())? {
                Navigation::PickFolder {} => {
                    return Ok(serde_json::json!({"path":picker::pick_folder()?}));
                }
                Navigation::PickFile {} => {
                    return Ok(serde_json::json!({"path":picker::pick_file()?}));
                }
                Navigation::PickExecutable {} => {
                    return Ok(serde_json::json!({"path":picker::pick_executable()?}));
                }
                Navigation::ResolveExecutable { executable } => {
                    return Ok(serde_json::json!({"path":crate::executable::resolve(&executable)}));
                }
                Navigation::Workspaces {} => return self.workspaces(),
                Navigation::SwitchWorkspace { path } => return self.switch_workspace(&path),
                Navigation::CreateWorkspace { name } => return self.create_workspace(&name),
                Navigation::ForgetWorkspace { path } => return self.forget_workspace(&path),
                Navigation::DeleteWorkspace {
                    path,
                    name,
                    replacement,
                } => {
                    drop(startup);
                    return self.delete_workspace(&path, &name, replacement.as_deref());
                }
                Navigation::SavedWorkspaces {} => return self.saved_workspaces(),
                Navigation::OpenWorkspace { path } => return self.open_workspace(&path),
                Navigation::SetLanguage { language } => {
                    language::save(&language)?;
                    return Ok(serde_json::json!({"language":language}));
                }
                Navigation::SetTheme { theme } => {
                    language::save_theme(&theme)?;
                    return Ok(serde_json::json!({"theme":theme}));
                }
                // DD-100: workspace signal preference and the last effective route.
                Navigation::SignalTransport {} => {
                    use crate::cooperative::route;
                    return Ok(serde_json::json!({
                        "preference": route::load_preference(startup.root()),
                        "last": route::last(startup.root()),
                    }));
                }
                Navigation::SetSignalTransport { preference } => {
                    crate::cooperative::route::save_preference(startup.root(), &preference)?;
                    return Ok(serde_json::json!({"preference":preference}));
                }
                Navigation::OpenRun {} => {
                    let mut controller = self
                        .controller
                        .lock()
                        .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?;
                    if controller.is_none() {
                        let dir = startup
                            .active_run()?
                            .ok_or(crate::ui::messages::text("native.run_is_not_ready_to_open"))?;
                        let lease = crate::leases::Lease::acquire(&dir.join("ui-window.lock"))?;
                        let next = Arc::new(Controller {
                            dir,
                            exe: startup.executable().to_owned(),
                            job: Mutex::new(None),
                        });
                        next.automatic(startup.auto_resume())?;
                        *controller = Some(next);
                        *self.run_lease.lock().map_err(|_| {
                            crate::ui::messages::text("native.window_lease_state_poisoned")
                        })? = Some(lease);
                    }
                }
                Navigation::Home {} => {
                    let mut controller = self
                        .controller
                        .lock()
                        .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?;
                    if let Some(active) = controller.as_ref()
                        && active.poll()?["state"] == "running"
                    {
                        return Err(crate::ui::messages::text(
                            "native.wait_for_the_accepted_room_action_before_returning",
                        )
                        .into());
                    }
                    *controller = None;
                    *self.run_lease.lock().map_err(|_| {
                        crate::ui::messages::text("native.window_lease_state_poisoned")
                    })? = None;
                }
            }
            return Ok(serde_json::json!({"opened":true}));
        }
        let controller = self
            .controller
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?
            .clone();
        let Some(controller) = controller else {
            return self
                .surface()?
                .ok_or(crate::ui::messages::text("native.no_active_surface"))?
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
                u64::try_from(now)
                    .map_err(|_| crate::ui::messages::text("native.briefing_clock_overflow"))?,
            );
        }
        if let Some(startup) = self.surface()? {
            startup.poll_job()?;
        }
        match request {
            Request::Briefing {} => unreachable!("briefing returned before polling mutations"),
            Request::Snapshot { icons } => controller.snapshot(icons),
            Request::Evidence {} => controller.evidence(),
            Request::Task { index } => controller.task(index),
            Request::Coordinator {} => controller.coordinator(),
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
        Ok(language::embed(HTML)
            .replace(
                "<link rel=\"stylesheet\" href=\"/ui.css\"><script src=\"/ui.js\" defer></script>",
                &format!("{policy}<style nonce=\"{nonce}\">{CSS}</style>"),
            )
            .replace(
                "</body>",
                &format!("<script nonce=\"{nonce}\">{I18N}\n{JS}</script></body>"),
            )
            .replace("{{APP_VERSION}}", env!("CARGO_PKG_VERSION")))
    }
}

mod controller;
use controller::*;

mod automatic;
mod update;
