use super::*;

impl Room {
    /// A native window holds one kernel lease per canonical run directory.
    pub fn open(dir: &Path, exe: &Path) -> Result<Self, String> {
        workflow::load(dir)?;
        let dir = dir.canonicalize().map_err(|e| e.to_string())?;
        let _ = workflow::sync_phase(&dir);
        let lease =
            crate::leases::Lease::acquire(&dir.join("ui-window.lock")).map_err(|error| {
                if error.starts_with("lease is held:") {
                    crate::ui::messages::text("native.agent_room_is_already_open_for_this_run")
                        .into()
                } else {
                    error
                }
            })?;
        let enabled = workflow::auto_resume_enabled(&dir);
        let events = Arc::new(events::EventBus::default());
        let event_bridge = events::ProcessEventBridge::new(Arc::clone(&events)).ok();
        let controller = Arc::new(Controller {
            dir,
            exe: exe.canonicalize().map_err(|e| e.to_string())?,
            job: Arc::new(Mutex::new(None)),
            event_endpoint: event_bridge.as_ref().map(|bridge| bridge.endpoint()),
            event_surface: events::EventSurface::AgentRoom,
        });
        controller.automatic(enabled)?;
        let listener = events::listener::RunChangeListener::spawn(
            controller.dir.join("report.json"),
            Arc::clone(&events),
        )
        .ok();
        Ok(Self {
            controller: Mutex::new(Some(controller)),
            startup: Mutex::new(None),
            workspace_watch: Arc::new(Mutex::new(std::sync::Weak::new())),
            run_lease: Mutex::new(None),
            window_lease: Mutex::new(lease),
            parked_workspaces: Mutex::new(std::collections::BTreeMap::new()),
            workspace_navigation: Mutex::new(()),
            events,
            _event_bridge: event_bridge,
            _listeners: listener.into_iter().collect(),
        })
    }

    pub fn startup(root: &Path, exe: &Path) -> Result<Self, String> {
        fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let _navigation = crate::leases::Lease::workspace(root)?;
        let lease = crate::leases::Lease::acquire(&root.join("ui-window.lock"))?;
        let events = Arc::new(events::EventBus::default());
        let event_bridge = events::ProcessEventBridge::new(Arc::clone(&events)).ok();
        let endpoint = event_bridge.as_ref().map(|bridge| bridge.endpoint());
        let startup = Arc::new(setup::Startup::open_with_event_endpoint(
            root, exe, endpoint,
        )?);
        let workspace_watch = Arc::new(Mutex::new(Arc::downgrade(&startup)));
        let watched = Arc::clone(&workspace_watch);
        let listener = events::listener::RunChangeListener::spawn_startup(
            move || {
                let startup = watched.lock().ok()?.upgrade()?;
                Some(events::listener::StartupWatch {
                    startup_path: startup.root().join("startup.json"),
                    report_path: startup.watched_run(),
                })
            },
            Arc::clone(&events),
        )
        .ok();
        Ok(Self {
            controller: Mutex::new(None),
            startup: Mutex::new(Some(startup)),
            workspace_watch,
            run_lease: Mutex::new(None),
            window_lease: Mutex::new(lease),
            parked_workspaces: Mutex::new(std::collections::BTreeMap::new()),
            workspace_navigation: Mutex::new(()),
            events,
            _event_bridge: event_bridge,
            _listeners: listener.into_iter().collect(),
        })
    }
}
