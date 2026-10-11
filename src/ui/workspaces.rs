//! DD-47: in-place workspace switching and the recent-workspace registry.
use super::{Room, setup};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

const LIMIT: usize = 50;

fn registry() -> Result<PathBuf, String> {
    let default = crate::desktop::workspace()?;
    Ok(default
        .parent()
        .ok_or("invalid default workspace")?
        .join("workspaces.json"))
}

fn stored() -> Vec<PathBuf> {
    registry()
        .ok()
        .and_then(|path| fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Existing absolute workspace directories, `current` first, without duplicates.
pub fn recent(current: &Path) -> Result<Vec<PathBuf>, String> {
    let stored = stored();
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
    write_registry(&recent(current)?)
}

fn write_registry(list: &[PathBuf]) -> Result<(), String> {
    let path = registry()?;
    let parent = path.parent().ok_or("invalid registry path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(list).map_err(|e| e.to_string())?;
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
        let registered: Vec<_> = recent(startup.root())?
            .into_iter()
            .map(|path| serde_json::json!({"name":name(&path),"path":path}))
            .collect();
        Ok(
            serde_json::json!({"current":startup.root(),"name":name(startup.root()),"registered":registered}),
        )
    }

    /// DD-48: a new workspace needs only a name; its folder lives in the data directory.
    pub(super) fn create_workspace(&self, title: &str) -> Result<serde_json::Value, String> {
        let title = title.trim();
        if title.is_empty() {
            return Err("enter a workspace name".into());
        }
        setup::validate_name(title)?;
        let slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        let slug = if slug.is_empty() {
            "workspace".into()
        } else {
            slug
        };
        let parent = registry()?
            .parent()
            .ok_or("invalid registry path")?
            .join("workspaces");
        let mut path = parent.join(&slug);
        let mut index = 2;
        while path.exists() {
            path = parent.join(format!("{slug}-{index}"));
            index += 1;
        }
        let observed = self
            .surface()?
            .ok_or("workspace surface unavailable")?
            .profiles()?;
        let result = self.switch_workspace(&path)?;
        let created = self.surface()?.ok_or("workspace surface unavailable")?;
        created.set_name(title)?;
        created.adopt_catalogs(&observed)?;
        Ok(result)
    }

    /// Workspaces saved in the data directory, including ones removed from the menu.
    pub(super) fn saved_workspaces(&self) -> Result<serde_json::Value, String> {
        let current = self.surface()?.ok_or("workspace surface unavailable")?;
        let registered = stored();
        let list: Vec<_> = saved()?
            .into_iter()
            .map(|path| {
                let listed = registered.contains(&path) || path == current.root();
                serde_json::json!({"name":name(&path),"registered":listed,"path":path})
            })
            .collect();
        Ok(serde_json::json!({"workspaces":list}))
    }

    /// Opens only a workspace saved in the data directory, never an arbitrary folder.
    pub(super) fn open_workspace(&self, path: &Path) -> Result<serde_json::Value, String> {
        let path = path
            .canonicalize()
            .map_err(|_| "that workspace is not saved in ~/.agentconductor")?;
        if !saved()?.contains(&path) {
            return Err("that folder is not a saved workspace in ~/.agentconductor".into());
        }
        self.switch_workspace(&path)
    }

    /// Removes a registry entry only; the folder and its evidence stay on disk.
    pub(super) fn forget_workspace(&self, path: &Path) -> Result<serde_json::Value, String> {
        let current = self.surface()?.ok_or("workspace surface unavailable")?;
        if path == current.root() {
            return Err("switch to another workspace before removing this one".into());
        }
        let list: Vec<PathBuf> = stored().into_iter().filter(|p| p != path).collect();
        write_registry(&list)?;
        Ok(serde_json::json!({"removed":true}))
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

/// Saved workspace name, falling back to the folder name.
fn name(path: &Path) -> String {
    fs::read(path.join("startup.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .and_then(|saved| saved["config"]["name"].as_str().map(str::to_owned))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| {
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
}

/// Folders under the data directory (`desktop`, `workspaces/*`) holding saved settings.
fn saved() -> Result<Vec<PathBuf>, String> {
    let data = registry()?
        .parent()
        .ok_or("invalid registry path")?
        .to_path_buf();
    let mut candidates = vec![data.join("desktop")];
    if let Ok(entries) = fs::read_dir(data.join("workspaces")) {
        let mut folders: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        folders.sort();
        candidates.extend(folders);
    }
    Ok(candidates
        .into_iter()
        .filter(|path| path.join("startup.json").is_file())
        .filter_map(|path| path.canonicalize().ok())
        .collect())
}
