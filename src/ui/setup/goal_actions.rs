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
        }
        Ok(
            serde_json::json!({"slug":slug,"goal":entry.goal,"run":run,"report":report,"evidence":evidence,"details":details}),
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
