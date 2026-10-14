//! Preserve owned active sessions while another workspace is shown.
use super::*;
use crate::ui::Controller;

pub(in crate::ui) struct ParkedWorkspace {
    pub(in crate::ui) startup: Arc<setup::Startup>,
    controller: Option<Arc<Controller>>,
    run_lease: Option<crate::leases::Lease>,
    window_lease: crate::leases::Lease,
}
fn active(startup: &setup::Startup, controller: Option<&Arc<Controller>>) -> bool {
    if startup.ensure_idle().is_err() {
        return true;
    }
    controller.is_some_and(|c| {
        c.busy()
            || crate::workflow::load(&c.dir).map_or(true, |r| {
                r.state == "running" || r.tasks.iter().any(|t| t.status == "running")
            })
    })
}
impl Room {
    pub(super) fn switch_preserving_session(
        &self,
        path: &Path,
    ) -> Result<serde_json::Value, String> {
        if !path.is_absolute() {
            return Err(
                crate::ui::messages::text("native.choose_an_absolute_workspace_folder").into(),
            );
        }
        let mut startup = self
            .startup
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?;
        let current = startup
            .as_ref()
            .ok_or(crate::ui::messages::text(
                "native.workspace_surface_unavailable",
            ))?
            .clone();
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
        let root = path.canonicalize().map_err(|e| e.to_string())?;
        if root == current.root() {
            return Ok(serde_json::json!({"switched":false}));
        }
        let _navigation = crate::leases::Lease::workspace(&root)?;
        let mut controller = self
            .controller
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?;
        let mut run_lease = self
            .run_lease
            .lock()
            .map_err(|_| crate::ui::messages::text("native.window_lease_state_poisoned"))?;
        let mut window_lease = self
            .window_lease
            .lock()
            .map_err(|_| crate::ui::messages::text("native.window_lease_state_poisoned"))?;
        let mut parked = self
            .parked_workspaces
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?;
        let keep = active(&current, controller.as_ref());
        let cached = parked.contains_key(&root);
        let next = match parked.remove(&root) {
            Some(session) => session,
            None => {
                let lease =
                    crate::leases::Lease::acquire(&root.join("ui-window.lock")).map_err(|e| {
                        if e.starts_with("lease is held:") {
                            crate::ui::messages::text(
                                "native.that_workspace_is_already_open_in_another_window",
                            )
                            .into()
                        } else {
                            e
                        }
                    })?;
                let next = setup::Startup::open(&root, current.executable())?;
                ParkedWorkspace {
                    startup: Arc::new(next),
                    controller: None,
                    run_lease: None,
                    window_lease: lease,
                }
            }
        };
        // Every mutation and job descriptor is already persisted in startup.json. Do not
        // overwrite it from an old snapshot while the CLI is updating its own run evidence.
        let remembered = remember(current.root()).and_then(|_| remember(&root));
        if let Err(error) = remembered {
            if cached {
                parked.insert(root, next);
            }
            return Err(error);
        }
        let previous = ParkedWorkspace {
            startup: startup.replace(next.startup).unwrap(),
            controller: std::mem::replace(&mut *controller, next.controller),
            run_lease: std::mem::replace(&mut *run_lease, next.run_lease),
            window_lease: std::mem::replace(&mut *window_lease, next.window_lease),
        };
        if keep {
            parked.insert(current.root().to_path_buf(), previous);
        }
        Ok(serde_json::json!({"switched":true,"current":root}))
    }
    pub(super) fn release_parked_for_delete(&self, path: &Path) -> Result<(), String> {
        let mut parked = self
            .parked_workspaces
            .lock()
            .map_err(|_| crate::ui::messages::text("native.surface_state_poisoned"))?;
        if let Some(session) = parked.get(path) {
            session.startup.ensure_idle()?;
            if active(&session.startup, session.controller.as_ref()) {
                return Err(crate::ui::messages::text(
                    "native.recover_or_stop_the_workspace_run_before_deleting",
                )
                .into());
            }
        }
        parked.remove(path);
        Ok(())
    }
}
