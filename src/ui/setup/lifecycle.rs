use super::*;
use std::collections::BTreeSet;
impl Startup {
    pub(super) fn store(&self, saved: &Saved) -> Result<(), String> {
        let path = self.broker.dir.join("startup.json");
        let previous = read::<Saved>(&path).ok();
        write(&path, saved)?;
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
            return Err("private cleanup binding mismatch".into());
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
            keep.insert(PathBuf::from(
                report["repo"]
                    .as_str()
                    .ok_or("run repository binding missing")?,
            ));
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
