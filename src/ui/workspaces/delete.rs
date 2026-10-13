//! Confirmed removal from the saved workspace list, with recoverable app data.
use super::*;
impl Room {
    pub(in crate::ui) fn delete_workspace(
        &self,
        path: &Path,
        confirmed_name: &str,
        replacement: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let current = self.surface()?.ok_or("workspace surface unavailable")?;
        let path = path.canonicalize().map_err(|e| e.to_string())?;
        let data = registry()?
            .parent()
            .ok_or("invalid registry path")?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let named = data.join("workspaces").canonicalize().ok();
        if (path != data.join("desktop") && named.as_deref() != path.parent())
            || !saved()?.contains(&path)
        {
            return Err("only a saved app-owned workspace can be deleted".into());
        }
        if name(&path) != confirmed_name {
            return Err("workspace name changed; review it before deleting".into());
        }
        let saved: serde_json::Value = serde_json::from_slice(
            &fs::read(path.join("startup.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        if let Some(project) = saved["config"]["repository"]
            .as_str()
            .filter(|s| !s.is_empty())
        {
            let project = Path::new(project)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if project.starts_with(&path) || path.starts_with(&project) {
                return Err("workspace data overlaps project sources; deletion refused".into());
            }
        }
        for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
            let run = entry.map_err(|e| e.to_string())?.path();
            if run.join("report.json").is_file() {
                let report = crate::workflow::load(&run)?;
                if report.state == "running" || report.tasks.iter().any(|t| t.status == "running") {
                    return Err("recover or stop the workspace run before deleting".into());
                }
            }
        }
        if path == current.root() {
            current.ensure_idle()?;
            // The fresh workspace takes the operator's name, if one was given.
            let title = replacement
                .map(str::trim)
                .filter(|t| !t.is_empty())
                .unwrap_or("New workspace");
            self.create_workspace(title)?;
            drop(current);
            return match self.delete_workspace(&path, confirmed_name, None) {
                Ok(result) => Ok(result),
                Err(error) => match self.switch_workspace(&path) {
                    Ok(_) => Err(error),
                    Err(restore) => Err(format!(
                        "{error}; original workspace retained at {}: {restore}",
                        path.display()
                    )),
                },
            };
        }
        let _navigation = crate::leases::Lease::workspace(&path)?;
        let _lease = crate::leases::Lease::acquire(&path.join("ui-window.lock"))?;
        let target = setup::Startup::open(&path, current.executable())?;
        target.ensure_idle()?;
        fs::write(path.join("workspace-deletion.json"), serde_json::to_vec_pretty(&serde_json::json!({"schema_version":1,"decision":"confirmed_archive","name":confirmed_name,"original":path})).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        drop(target);
        // A sibling rename retains logs and history without traversing project files.
        let trash = data.join("workspace-trash");
        fs::create_dir_all(&trash).map_err(|e| e.to_string())?;
        if !crate::folder::ordinary(&fs::symlink_metadata(&trash).map_err(|e| e.to_string())?) {
            return Err("workspace recovery folder must be an ordinary directory".into());
        }
        let archived = trash.join(crate::native_guard::session_id()?);
        // Windows cannot rename a folder while its UI lock handle is open.
        drop(_lease);
        fs::rename(&path, &archived).map_err(|e| e.to_string())?;
        let list: Vec<_> = stored().into_iter().filter(|p| p != &path).collect();
        if let Err(error) = write_registry(&list) {
            return match fs::rename(&archived, &path) {
                Ok(()) => Err(error),
                Err(rollback) => Err(format!(
                    "{error}; workspace retained at {}: {rollback}",
                    archived.display()
                )),
            };
        }
        Ok(serde_json::json!({"deleted":true,"archived":archived}))
    }
}
