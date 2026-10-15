//! Keep native request routing separate from window/surface lifetime.
use super::*;
impl Room {
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
        if kind["kind"] == "mcp_source_suggestions" {
            let goal = kind["goal"]
                .as_str()
                .ok_or("source suggestion request requires a goal")?;
            return Ok(serde_json::json!({
                "references": source_references::references(goal),
                "suggestions": source_references::suggestions(goal)
            }));
        }
        // User request 2026-10-14: the workspace page and the Agent Room stay open side by side.
        // Agent Room requests carry "room": true and go to the run; all others to the workspace.
        let from_room = kind["room"] == true;
        let unwrapped;
        let bytes = if from_room {
            let mut packet = kind.clone();
            if let Some(fields) = packet.as_object_mut() {
                fields.remove("room");
            }
            unwrapped = serde_json::to_vec(&packet).map_err(|e| e.to_string())?;
            unwrapped.as_slice()
        } else {
            bytes
        };
        if kind["kind"] == "manual_document" {
            let html = match kind["language"].as_str() {
                Some("en") => include_str!("../../docs/user-manual.html"),
                Some("ko") => include_str!("../../docs/user-manual.ko.html"),
                _ => return Err("unsupported manual language".into()),
            };
            return Ok(serde_json::json!({"html":html}));
        }
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
            let _navigation = if matches!(
                kind["kind"].as_str(),
                Some(
                    "switch_workspace" | "create_workspace" | "open_workspace" | "delete_workspace"
                )
            ) {
                Some(
                    self.workspace_navigation
                        .lock()
                        .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?,
                )
            } else {
                None
            };
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
                Navigation::SwitchWorkspace { path } => {
                    return self.switch_workspace(&path);
                }
                Navigation::CreateWorkspace { name } => {
                    return self.create_workspace(&name);
                }
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
                Navigation::OpenWorkspace { path } => {
                    return self.open_workspace(&path);
                }
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
                    // A room kept open for an earlier run is replaced by the workspace's current one.
                    let active = startup.active_run()?;
                    let stale = controller
                        .as_ref()
                        .is_some_and(|open| active.as_ref().is_some_and(|run| *run != open.dir));
                    if controller.is_none() || stale {
                        let dir = active
                            .ok_or(crate::ui::messages::text("native.run_is_not_ready_to_open"))?;
                        *self.run_lease.lock().map_err(|_| {
                            crate::ui::messages::text("native.window_lease_state_poisoned")
                        })? = None;
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
                // Returning to the workspace keeps the Agent Room open, so its progress stays
                // live and switching back is instant (user request 2026-10-14).
                Navigation::Home {} => {}
            }
            let run = self
                .controller
                .lock()
                .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?
                .as_ref()
                .map(|open| open.dir.clone());
            return Ok(serde_json::json!({"opened":true,"run":run}));
        }
        if !from_room {
            let changing_goal = matches!(kind["kind"].as_str(), Some("new_goal" | "select_goal"));
            let _goal_navigation = if changing_goal {
                Some(
                    self.workspace_navigation
                        .lock()
                        .map_err(|_| messages::text("native.surface_state_poisoned"))?,
                )
            } else {
                None
            };
            if let Some(surface) = self.surface()? {
                if changing_goal
                    && self
                        .controller
                        .lock()
                        .map_err(|_| messages::text("native.surface_state_poisoned"))?
                        .as_ref()
                        .is_some_and(|run| run.busy())
                {
                    return Err(
                        messages::text("native.finish_or_stop_run_before_changing_goals").into(),
                    );
                }
                let result = surface.dispatch(bytes);
                if changing_goal && result.as_ref().is_ok_and(|value| value["changed"] == true) {
                    *self
                        .controller
                        .lock()
                        .map_err(|_| messages::text("native.surface_state_poisoned"))? = None;
                    *self
                        .run_lease
                        .lock()
                        .map_err(|_| messages::text("native.window_lease_state_poisoned"))? = None;
                }
                return result;
            }
        }
        let controller = self
            .controller
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?
            .clone();
        let Some(controller) = controller else {
            return Err(crate::ui::messages::text("native.no_active_surface").into());
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
}
