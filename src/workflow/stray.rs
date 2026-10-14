//! New files an agent left outside every task's owned paths in a project without its own Git
//! (for example a CLI saving its plans under .aipro/plans/) are moved out of the project into
//! the run folder before integration, and recorded. Files the operator already had are never
//! touched: only paths that are new since the run's baseline are moved.
use super::*;

/// New, untracked paths outside every owned scope (repository-relative, `/`-separated).
pub(super) fn stray_files(git: &Repository, plan: &Plan) -> Result<Vec<String>, String> {
    if !git.is_external() {
        return Ok(vec![]);
    }
    let scopes = scopes(plan);
    let listed = git.text(&["ls-files", "--others", "--exclude-standard", "-z"])?;
    Ok(listed
        .split('\0')
        .filter(|path| !path.is_empty())
        // Secrets, build outputs and app folders are never project deliverables to move.
        .filter(|path| !path.split('/').any(crate::folder::excluded))
        .filter(|path| {
            !scopes
                .iter()
                .any(|scope| crate::plan::overlaps(path, scope.trim_end_matches('/')))
        })
        .map(str::to_owned)
        .collect())
}

/// Moves stray files into `<run>/stray-files/` and records what moved.
pub(super) fn move_stray_files(
    git: &Repository,
    plan: &Plan,
    dir: &Path,
    store: &mut Store,
) -> Result<Vec<String>, String> {
    let stray = stray_files(git, plan)?;
    if stray.is_empty() {
        return Ok(stray);
    }
    let target = dir.join("stray-files");
    for path in &stray {
        let from = git.root.join(path);
        let to = target.join(path);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        if fs::rename(&from, &to).is_err() {
            fs::copy(&from, &to).map_err(|e| e.to_string())?;
            fs::remove_file(&from).map_err(|e| e.to_string())?;
        }
        // Remove directories the move left empty (for example .aipro/plans/).
        let mut parent = from.parent();
        while let Some(folder) = parent.filter(|p| *p != git.root.as_path()) {
            if fs::remove_dir(folder).is_err() {
                break;
            }
            parent = folder.parent();
        }
    }
    store.record(
        "stray_files_moved",
        &serde_json::json!({"paths": stray, "to": target}),
        "Agents created files outside every owned path; they were moved out of the project",
    )?;
    Ok(stray)
}
