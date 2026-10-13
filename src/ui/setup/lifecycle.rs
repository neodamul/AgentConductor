use super::*;
use std::collections::BTreeSet;
impl Startup {
    pub(super) fn store(&self, saved: &Saved) -> Result<(), String> {
        let path = self.broker.dir.join("startup.json");
        let previous = read::<Saved>(&path).ok();
        write(&path, saved)?;
        // DD-109: the agent list is shared by every app-owned workspace.
        if let Some(master) = super::master::path(&self.broker.dir) {
            super::master::store(&master, &saved.config.profiles)?;
        }
        if let Some(old) = previous.and_then(|s| s.proposal)
            && old.phase != "starting"
            && old.workspace != saved.proposal.as_ref().and_then(|p| p.workspace.clone())
            && let Some(workspace) = old.workspace
            && let Err(error) = self.remove_unused_history(&workspace)
        {
            eprintln!("Private history cleanup deferred: {error}");
        }
        Ok(())
    }
    fn remove_unused_history(&self, path: &Path) -> Result<(), String> {
        if !path.starts_with(self.broker.dir.join("folder-history")) {
            return Ok(());
        }
        if !path.exists() {
            return Ok(());
        }
        let history = self
            .broker
            .dir
            .join("folder-history")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        if !crate::folder::ordinary(&metadata) || path.parent() != Some(history.as_path()) {
            return Ok(());
        }
        let Some(source) = crate::folder::load(&path)? else {
            return Ok(());
        };
        if source.workspace != path {
            return Err(
                crate::ui::messages::text("native.private_cleanup_binding_mismatch").into(),
            );
        }
        let _lease = crate::leases::WorkspaceLease::acquire(&path, &self.broker.dir)?;
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    }
    pub(super) fn prune_history(&self, saved: &Saved) -> Result<(), String> {
        let history = self.broker.dir.join("folder-history");
        if !history.is_dir() {
            return Ok(());
        }
        let mut keep = BTreeSet::new();
        if let Some(path) = saved.proposal.as_ref().and_then(|p| p.workspace.clone()) {
            keep.insert(path);
        }
        for entry in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_name().to_string_lossy().starts_with("run-") {
                continue;
            }
            let report = entry.path().join("report.json");
            if !report.is_file() {
                continue;
            }
            // Unknown run ownership protects all copies rather than guessing.
            let report: serde_json::Value =
                serde_json::from_slice(&fs::read(report).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            keep.insert(PathBuf::from(report["repo"].as_str().ok_or(
                crate::ui::messages::text("native.run_repository_binding_missing"),
            )?));
        }
        for entry in fs::read_dir(history).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_name().to_string_lossy().starts_with("folder-")
                && !keep.contains(&entry.path())
                && let Err(error) = self.remove_unused_history(&entry.path())
            {
                eprintln!("Private history cleanup deferred: {error}");
            }
        }
        Ok(())
    }
}

impl Startup {
    fn cache_protected(&self, saved: &Saved) -> Result<Vec<PathBuf>, String> {
        let mut keep = Vec::new();
        if let Some(job) = &saved.quota_job {
            keep.push(
                self.quota_broker
                    .dir
                    .join(format!("coordinator-ui-{}", job.id)),
            );
        }
        if let Some(path) = &saved.metadata_input {
            keep.push(path.clone());
        }
        if !saved.config.repository.as_os_str().is_empty() {
            keep.push(
                saved
                    .config
                    .repository
                    .canonicalize()
                    .map_err(|e| e.to_string())?,
            );
        }
        if let Some(proposal) = &saved.proposal {
            keep.extend([proposal.directory.clone(), proposal.run.clone()]);
            if let Some(workspace) = &proposal.workspace {
                keep.push(workspace.clone());
            }
        }
        for item in fs::read_dir(&self.broker.dir).map_err(|e| e.to_string())? {
            let item = item.map_err(|e| e.to_string())?;
            if item.file_name().to_string_lossy().starts_with("run-")
                && item.path().join("report.json").is_file()
            {
                let report = crate::workflow::load(&item.path())?;
                keep.extend([report.repo, report.verification_cwd]);
                if let Some(folder) = report.folder {
                    keep.push(folder.original);
                }
                for task in report.tasks {
                    keep.extend([task.cwd, task.artifacts]);
                }
            }
        }
        Ok(keep)
    }
    pub(super) fn cache_preview(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        super::super::cache::preview(&self.broker.dir, &self.cache_protected(saved)?)
    }
    pub(super) fn clear_cache(
        &self,
        saved: &Saved,
        expected: Option<&str>,
        keep: usize,
    ) -> Result<serde_json::Value, String> {
        if self.job(saved)?["state"] == "running"
            || self.quota_job(saved)?["state"] == "running"
            || !saved.metadata_queue.is_empty()
        {
            return Err(crate::ui::messages::text(
                "native.wait_for_active_planning_or_metadata_before_clearing",
            )
            .into());
        }
        self.ensure_no_live_jobs()?;
        let result = super::super::cache::clear(
            &self.broker.dir,
            &self.cache_protected(saved)?,
            expected,
            keep,
        )?;
        if result["removed"].as_u64().unwrap_or(0) > 0 {
            crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record(
                "cache_cleanup",
                &result,
                crate::ui::messages::text(
                    "native.removed_disposable_metadata_cache_reviewed_evidence_retained",
                ),
            )?;
        }
        Ok(result)
    }
}
