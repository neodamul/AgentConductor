//! GitHub #331: what in this workspace stops an app update, as named work and not as one refusal.
use super::*;
use crate::ui::update_guard::{Blocker, job_label};

impl Startup {
    pub(crate) fn update_blockers(&self) -> Vec<Blocker> {
        let saved = match self.saved.lock() {
            Ok(saved) => saved,
            Err(_) => {
                return vec![Blocker::Unknown {
                    what: "workspace settings".into(),
                    error: messages::text("native.startup_settings_poisoned").into(),
                }];
            }
        };
        let workspace = if saved.config.name.trim().is_empty() {
            self.broker.dir.display().to_string()
        } else {
            saved.config.name.clone()
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis() as u64);
        let language = crate::ui::language::load();
        let mut out = Vec::new();
        for (job, default_kind) in [
            (self.job(&saved), "plan"),
            (self.quota_job(&saved), "quota"),
            (self.chat_job(&saved), "chat"),
        ] {
            match job {
                Ok(job) if job["state"] == "running" => out.push(Blocker::Active {
                    workspace: workspace.clone(),
                    what: job_label(&language, job["kind"].as_str().unwrap_or(default_kind)),
                    elapsed_ms: job["started_ms"].as_u64().map(|at| now.saturating_sub(at)),
                }),
                Ok(_) => {}
                Err(error) => out.push(Blocker::Unknown {
                    what: format!("{workspace}: {}", job_label(&language, default_kind)),
                    error,
                }),
            }
        }
        drop(saved);
        out.extend(self.live_job_blockers(&language, &workspace));
        out
    }

    /// The retained startup jobs whose process is still there (running work), apart from ones that
    /// cannot be read or proven gone (a failed check).
    fn live_job_blockers(&self, language: &str, workspace: &str) -> Vec<Blocker> {
        let entries = match fs::read_dir(&self.broker.dir) {
            Ok(entries) => entries,
            Err(e) => {
                return vec![Blocker::Unknown {
                    what: self.broker.dir.display().to_string(),
                    error: e.to_string(),
                }];
            }
        };
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("coordinator-ui-") {
                continue;
            }
            let owner: crate::process::Ownership =
                match read(&entry.path().join("process/owner.json")) {
                    Ok(owner) => owner,
                    Err(error) => {
                        out.push(Blocker::Unknown { what: name, error });
                        continue;
                    }
                };
            // Successful startup commands have reaped their workers; old PIDs can be reused.
            if owner.phase == "terminal"
                && owner.exit_code == Some(0)
                && owner.pid.is_some_and(|id| id > 0 && id <= i32::MAX as u32)
                && owner.process_group == owner.pid.map(|id| id as i32)
            {
                continue;
            }
            if let Err(error) = owner.ensure_quiescent() {
                out.push(if error.contains("still present") {
                    Blocker::Active {
                        workspace: workspace.into(),
                        what: format!("{} ({name})", job_label(language, "start")),
                        elapsed_ms: None,
                    }
                } else {
                    Blocker::Unknown { what: name, error }
                });
            }
        }
        out
    }
}
