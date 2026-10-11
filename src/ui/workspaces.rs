//! DD-47: in-place workspace switching and the recent-workspace registry.
use super::{Room, setup};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

const LIMIT: usize = 20;

fn registry() -> Result<PathBuf, String> {
    let default = crate::desktop::workspace()?;
    Ok(default
        .parent()
        .ok_or("invalid default workspace")?
        .join("workspaces.json"))
}

/// Existing absolute workspace directories, `current` first, without duplicates.
pub fn recent(current: &Path) -> Result<Vec<PathBuf>, String> {
    let stored: Vec<PathBuf> = registry()
        .ok()
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    let mut list = vec![current.to_path_buf()];
    for path in stored {
        if path.is_absolute() && path.is_dir() && !list.contains(&path) {
            list.push(path);
        }
    }
    list.truncate(LIMIT);
    Ok(list)
}

pub fn remember(current: &Path) -> Result<(), String> {
    let path = registry()?;
    let parent = path.parent().ok_or("invalid registry path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(&recent(current)?).map_err(|e| e.to_string())?;
    let pending = parent.join(format!(
        "workspaces.{}.pending",
        crate::native_guard::session_id()?
    ));
    fs::write(&pending, bytes).map_err(|e| e.to_string())?;
    fs::rename(&pending, &path).map_err(|e| e.to_string())
}

impl Room {
    pub(super) fn workspaces(&self) -> Result<serde_json::Value, String> {
        let startup = self.surface()?.ok_or("workspace surface unavailable")?;
        Ok(serde_json::json!({"current":startup.root(),"recent":recent(startup.root())?}))
    }

    /// Acquire the target lease and state first; the current workspace stays open on failure.
    pub(super) fn switch_workspace(&self, path: &Path) -> Result<serde_json::Value, String> {
        if !path.is_absolute() {
            return Err("choose an absolute workspace folder".into());
        }
        let current = self.surface()?.ok_or("workspace surface unavailable")?;
        if self
            .controller
            .lock()
            .map_err(|_| "surface state poisoned")?
            .is_some()
        {
            return Err("return to the workspace before switching".into());
        }
        current.ensure_idle()?;
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
        let root = path.canonicalize().map_err(|e| e.to_string())?;
        if root == current.root() {
            return Ok(serde_json::json!({"switched":false}));
        }
        let lease = crate::leases::Lease::acquire(&root.join("ui-window.lock")).map_err(|e| {
            if e.starts_with("lease is held:") {
                "that workspace is already open in another window".into()
            } else {
                e
            }
        })?;
        let next = setup::Startup::open(&root, current.executable())?;
        *self.startup.lock().map_err(|_| "surface state poisoned")? = Some(Arc::new(next));
        *self
            .window_lease
            .lock()
            .map_err(|_| "window lease state poisoned")? = lease;
        // Keep the workspace being left reachable from the menu.
        remember(current.root())?;
        remember(&root)?;
        Ok(serde_json::json!({"switched":true,"current":root}))
    }
}
