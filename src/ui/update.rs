//! DD-135: update requests from the page. Installing ends this process, and with it every
//! worker the app started, so it is refused while any run or app job is active.
use super::*;

impl Room {
    pub(super) fn update(&self, kind: &str) -> Result<serde_json::Value, String> {
        match kind {
            "check_update" => crate::update::check_async(),
            "apply_update" => {
                self.ensure_update_idle()?;
                crate::update::apply_async()?;
            }
            _ => {}
        }
        Ok(crate::update::status())
    }

    pub(super) fn update_source(request: &serde_json::Value) -> Result<serde_json::Value, String> {
        if request["kind"] == "set_update_settings" {
            crate::update_source::save(request["repository"].as_str().unwrap_or_default())?;
            if let Some(channel) = request["channel"].as_str() {
                crate::update_source::save_channel(channel)?;
            }
        }
        Ok(crate::update_source::settings())
    }

    /// GitHub #331: running work (workspace, kind, time) and failed checks are named apart; a run
    /// whose coordinator is gone does not count.
    fn ensure_update_idle(&self) -> Result<(), String> {
        use crate::ui::update_guard::{Blocker, job_label, message, run_blocker};
        let language = crate::ui::language::load();
        let mut blockers = Vec::new();
        match self.surface() {
            Ok(Some(startup)) => blockers.extend(startup.update_blockers()),
            Ok(None) => {}
            Err(error) => blockers.push(Blocker::Unknown {
                what: "workspace".into(),
                error,
            }),
        }
        match self.controller.lock() {
            Ok(slot) => {
                if let Some(controller) = slot.clone() {
                    match controller.poll() {
                        Ok(job) if job["state"] == "running" => blockers.push(Blocker::Active {
                            workspace: controller
                                .dir
                                .parent()
                                .and_then(|p| p.file_name())
                                .map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                            what: job_label(&language, job["kind"].as_str().unwrap_or("")),
                            elapsed_ms: None,
                        }),
                        Ok(_) => {}
                        Err(error) => blockers.push(Blocker::Unknown {
                            what: "run job".into(),
                            error,
                        }),
                    }
                }
            }
            Err(_) => blockers.push(Blocker::Unknown {
                what: "run job".into(),
                error: crate::ui::messages::text("native.surface_state_poisoned").into(),
            }),
        }
        // Runs of every saved workspace, not only the open one.
        match saved_runs() {
            Ok(runs) => {
                for (name, run) in runs {
                    blockers.extend(run_blocker(&language, &name, &run));
                }
            }
            Err(error) => blockers.push(Blocker::Unknown {
                what: "workspace list".into(),
                error,
            }),
        }
        if blockers.is_empty() {
            Ok(())
        } else {
            Err(message(&language, &blockers))
        }
    }
}

/// Every run folder (with its workspace name) of every saved workspace.
pub(super) fn saved_runs() -> Result<Vec<(String, PathBuf)>, String> {
    let mut runs = Vec::new();
    for workspace in workspaces::saved()? {
        let Ok(entries) = fs::read_dir(&workspace) else {
            continue;
        };
        let name = workspace
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        for run in entries.flatten().map(|e| e.path()) {
            if run.join("report.json").is_file() {
                runs.push((name.clone(), run));
            }
        }
    }
    Ok(runs)
}
