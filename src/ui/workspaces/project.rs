//! One active workspace owns a project; legacy duplicate data is retained.
use super::*;
pub(in crate::ui) fn binding_lease(project: &Path) -> Result<crate::leases::Lease, String> {
    crate::leases::Lease::project_binding(project)
}
pub(in crate::ui) fn validate_owner(project: &Path, current: &Path) -> Result<(), String> {
    if project.as_os_str().is_empty() {
        return Ok(());
    }
    let project = project.canonicalize().map_err(|e| e.to_string())?;
    let current = current.canonicalize().map_err(|e| e.to_string())?;
    let mut candidates = saved()?;
    candidates.extend(stored());
    candidates.push(current.clone());
    let mut owners = std::collections::BTreeMap::new();
    for path in candidates {
        let Ok(path) = path.canonicalize() else {
            continue;
        };
        let Ok(bytes) = fs::read(path.join("startup.json")) else {
            continue;
        };
        let saved: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let Some(folder) = saved["config"]["repository"]
            .as_str()
            .filter(|v| !v.is_empty())
        else {
            continue;
        };
        if Path::new(folder).canonicalize().ok().as_ref() == Some(&project) {
            let created = fs::metadata(&path)
                .and_then(|v| v.created())
                .unwrap_or(std::time::UNIX_EPOCH);
            owners.insert((created, path.clone()), name(&path));
        }
    }
    if let Some(((_, owner), name)) = owners.first_key_value()
        && owner != &current
    {
        return Err(crate::ui::messages::render(
            "native.project_belongs_to_workspace",
            &[("name", name), ("path", &owner.display().to_string())],
        ));
    }
    Ok(())
}
