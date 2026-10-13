//! Restore a rejected direct-folder candidate without overwriting later manual edits.
use super::*;
pub fn restore_rejected(
    git: &Repository,
    report: &crate::workflow::Report,
    run: &Path,
) -> Result<(), String> {
    if !git.is_external() || report.plan.workspace_mode != crate::plan::WorkspaceMode::Shared {
        return Ok(());
    }
    let candidate = report.candidate.as_ref().ok_or("no verified candidate")?;
    if git.head()? != report.base || git.branch()? != report.branch {
        return Err(crate::ui::messages::text("folder.workspace_binding_changed").into());
    }
    let scopes = report
        .plan
        .tasks
        .iter()
        .flat_map(|t| t.paths.iter().cloned())
        .collect::<Vec<_>>();
    if git
        .snapshot(&report.base, &scopes, &run.join("reject.index"))?
        .tree
        != candidate.tree
    {
        return Err(crate::ui::messages::text("folder.owned_files_changed").into());
    }
    let source = Source {
        original: report.repo.clone(),
        workspace: report.repo.clone(),
        base: candidate.commit.clone(),
        files: scan(&report.repo)?,
    };
    let original = crate::git::Snapshot {
        commit: report.base.clone(),
        tree: git.text(&["rev-parse", &format!("{}^{{tree}}", report.base)])?,
        changed: candidate.changed.clone(),
    };
    let journal = run.join("workspace-rejection");
    fs::create_dir(&journal).map_err(|e| e.to_string())?;
    source.apply(git, &original, &journal)?;
    git.clean()
}
