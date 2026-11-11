//! R-211 / DD-223: read Goal history without selecting it; delete only verified owned data.
use super::*;
mod deletion;

impl Startup {
    fn goal_entry<'a>(
        &self,
        saved: &'a Saved,
        slug: &str,
    ) -> Result<&'a crate::slugs::Slug, String> {
        crate::slugs::validate(slug)?;
        saved
            .config
            .slugs
            .iter()
            .find(|g| g.slug == slug)
            .ok_or_else(|| messages::text("native.goal_is_not_registered").into())
    }

    fn goal_root(&self, saved: &Saved, slug: &str) -> Result<PathBuf, String> {
        self.goal_entry(saved, slug)?;
        crate::ui::workspaces::project::validate_owner(&saved.config.repository, &self.broker.dir)?;
        let parent = if saved.config.repository.as_os_str().is_empty() {
            &self.broker.dir
        } else {
            &saved.config.repository
        };
        let parent = parent.canonicalize().map_err(|e| e.to_string())?;
        let root = parent.join(slug);
        let meta = fs::symlink_metadata(&root).map_err(|e| e.to_string())?;
        if !meta.is_dir() || !crate::folder::ordinary(&meta) {
            return Err("Goal folder must be an ordinary directory, not an alias".into());
        }
        let canonical = root.canonicalize().map_err(|e| e.to_string())?;
        if canonical != root || canonical.parent() != Some(parent.as_path()) {
            return Err("Goal folder is outside its registered parent".into());
        }
        let marker = bytes(&root.join(crate::slugs::MARKER))?;
        if marker != b"AgentConductor goal work tree; not project source.\n" {
            return Err("Goal folder ownership marker is invalid".into());
        }
        Ok(root)
    }

    pub(super) fn goal_folder(
        &self,
        saved: &Saved,
        slug: &str,
        open: bool,
    ) -> Result<serde_json::Value, String> {
        let root = self.goal_root(saved, slug)?;
        if open {
            reveal(&root)?;
        }
        Ok(
            serde_json::json!({"slug":slug,"goal":self.goal_entry(saved,slug)?.goal,"root":root,"opened":open}),
        )
    }

    fn verified_goal_run(&self, saved: &Saved, slug: &str, run: &Path) -> Result<PathBuf, String> {
        let goal = self.goal_entry(saved, slug)?;
        let meta = fs::symlink_metadata(run).map_err(|e| e.to_string())?;
        let canonical = run.canonicalize().map_err(|e| e.to_string())?;
        if !meta.is_dir()
            || !crate::folder::ordinary(&meta)
            || canonical.parent() != Some(self.broker.dir.as_path())
            || !canonical
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("run-"))
        {
            return Err("Goal run is outside its owning Workspace".into());
        }
        let report = workflow::load(&canonical)?;
        let project = saved.config.repository.canonicalize().ok();
        let bound = report
            .folder
            .as_ref()
            .map_or(&report.repo, |f| &f.original)
            .canonicalize()
            .ok();
        if report.plan.goal != goal.goal || (project.is_some() && project != bound) {
            return Err(messages::text("native.goal_run_binding_changed").into());
        }
        Ok(canonical)
    }

    fn selected_goal_run(&self, saved: &Saved, slug: &str) -> Result<Option<PathBuf>, String> {
        let entry = self.goal_entry(saved, slug)?;
        let run = if saved.config.active_slug.as_deref() == Some(slug) {
            saved.last_run.as_ref().or(entry.last_run.as_ref())
        } else {
            entry.last_run.as_ref()
        };
        run.map(|run| self.verified_goal_run(saved, slug, run))
            .transpose()
    }

    pub(super) fn goal_result(
        &self,
        saved: &Saved,
        slug: &str,
    ) -> Result<serde_json::Value, String> {
        let entry = self.goal_entry(saved, slug)?;
        let run = self.selected_goal_run(saved, slug)?;
        let mut report = serde_json::Value::Null;
        let mut evidence = serde_json::Value::Null;
        let mut details = Vec::new();
        let mut phase_results = Vec::new();
        if let Some(run) = &run {
            let controller = Controller {
                dir: run.clone(),
                exe: self.broker.exe.clone(),
                job: Mutex::new(None),
            };
            evidence = controller.evidence()?;
            report = evidence["report"].clone();
            for index in 0..report["tasks"]
                .as_array()
                .ok_or("Invalid Goal result tasks")?
                .len()
            {
                details.push(controller.task(index)?);
            }
            if workflow::revision(&workflow::load(run)?)? != evidence["revision"] {
                return Err("Goal result changed while collecting evidence; refresh".into());
            }
            let root = crate::slugs::root(
                &saved.config.repository,
                &self.broker.dir,
                &saved.config.name,
                slug,
            );
            let root = root.canonicalize().map_err(|e| e.to_string())?;
            for item in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                let Some(name) = item.file_name().to_str().map(str::to_owned) else {
                    continue;
                };
                let Some(run_name) = name.strip_suffix(".slug-phase.json") else {
                    continue;
                };
                let meta = fs::symlink_metadata(item.path()).map_err(|e| e.to_string())?;
                if !meta.is_file() || !crate::folder::ordinary(&meta) {
                    continue;
                }
                let marker: serde_json::Value =
                    serde_json::from_slice(&fs::read(item.path()).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                if PathBuf::from(marker["root"].as_str().ok_or("invalid phase link")?)
                    .canonicalize()
                    .ok()
                    .as_ref()
                    != Some(&root)
                {
                    continue;
                }
                let Some(phase) = marker["phase"].as_str() else {
                    continue;
                };
                let Ok(phase_run) =
                    self.verified_goal_run(saved, slug, &self.broker.dir.join(run_name))
                else {
                    continue;
                };
                let phase_report = workflow::load(&phase_run)?;
                let controller = Controller {
                    dir: phase_run.clone(),
                    exe: self.broker.exe.clone(),
                    job: Mutex::new(None),
                };
                let phase_evidence = controller.evidence()?;
                if workflow::revision(&phase_report)? != phase_evidence["revision"] {
                    return Err("Goal Phase changed while collecting evidence; refresh".into());
                }
                let mut phase_details = Vec::new();
                for index in 0..phase_report.tasks.len() {
                    phase_details.push(controller.task(index)?);
                }
                phase_results.push(serde_json::json!({
                    "phase": phase,
                    "state": phase_report.state,
                    "report": phase_evidence["report"],
                    "evidence": phase_evidence,
                    "details": phase_details,
                }));
            }
            let order: Vec<String> = crate::slug_progress::progress(&root)["phases"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|phase| phase["id"].as_str().map(str::to_owned))
                .collect();
            phase_results.sort_by_key(|result| {
                (
                    order
                        .iter()
                        .position(|id| id == result["phase"].as_str().unwrap_or(""))
                        .unwrap_or(usize::MAX),
                    result["phase"].as_str().unwrap_or("").to_owned(),
                )
            });
        }
        Ok(
            serde_json::json!({"slug":slug,"goal":entry.goal,"run":run,"report":report,"evidence":evidence,"details":details,"phase_results":phase_results}),
        )
    }
}

fn reveal(root: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = crate::subprocess::command("/usr/bin/open");
    #[cfg(windows)]
    let mut command = crate::subprocess::command("explorer.exe");
    #[cfg(not(any(windows, target_os = "macos")))]
    let mut command = crate::subprocess::command("xdg-open");
    command
        .arg(crate::git::git_path(root.as_os_str()))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot open Goal folder: {e}"))?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
