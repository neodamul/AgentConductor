//! DD-109: the machine-wide agent CLI list. App-owned workspaces (under ~/.agentconductor)
//! open with it and write it back whenever their agent list changes; deleting a workspace
//! never touches it. Workspace references to agents it lacks are kept for reassignment.
use super::*;

/// `~/.agentconductor/agents.json` when `workspace` is an app-owned workspace.
pub(crate) fn path(workspace: &Path) -> Option<PathBuf> {
    let root = crate::desktop::workspace()
        .ok()?
        .parent()?
        .canonicalize()
        .ok()?;
    let workspace = workspace.canonicalize().ok()?;
    (workspace != root && workspace.starts_with(&root)).then(|| root.join("agents.json"))
}

pub(crate) fn load(path: &Path) -> Option<Vec<AgentProfile>> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

/// Opens with the master list, carrying over this workspace's private values; the first
/// app-owned workspace with agents seeds it.
pub(super) fn adopt(workspace: &Path, profiles: &mut Vec<AgentProfile>) {
    if let Some(path) = path(workspace) {
        match load(&path) {
            Some(mut master) if validate_profiles(&master).is_ok() => {
                profiles::carry_over(&mut master, profiles);
                *profiles = master;
            }
            _ if !profiles.is_empty() => {
                let _ = store(&path, profiles);
            }
            _ => {}
        }
    }
    // Profiles saved before capabilities were derived get theirs on open.
    profiles::derive_permissions(profiles);
}

pub(super) fn store(path: &Path, profiles: &[AgentProfile]) -> Result<(), String> {
    // A new empty workspace does not create an empty master list.
    if profiles.is_empty() && !path.exists() {
        return Ok(());
    }
    if load(path)
        .as_deref()
        .map(serde_json::to_value)
        .transpose()
        .ok()
        .flatten()
        == serde_json::to_value(profiles).ok()
    {
        return Ok(());
    }
    let staged = path.with_extension("json.tmp");
    fs::write(
        &staged,
        serde_json::to_vec_pretty(profiles).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(staged, path).map_err(|e| e.to_string())
}
