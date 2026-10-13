//! DD-62: link a run to one slug phase so its lifecycle moves the phase's task files.
use super::*;

const LINK: &str = "slug-phase.json";

/// Records the link after the run directory exists, then syncs the current state.
pub fn link_phase(dir: &Path, slug_root: &Path, phase: &str) -> Result<(), String> {
    let link = serde_json::json!({"root": slug_root, "phase": phase});
    fs::write(
        dir.join(LINK),
        serde_json::to_vec(&link).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    sync_phase(dir)
}

/// The linked goal folder, following a move to `<project>/<slug>` (user decision
/// 2026-10-13) from `<project>/.agentconductor/<slug>` or an interim `<project>/<name>/<slug>`.
fn linked(dir: &Path) -> Result<Option<(PathBuf, String)>, String> {
    let Ok(bytes) = fs::read(dir.join(LINK)) else {
        return Ok(None);
    };
    let link: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let mut root = PathBuf::from(link["root"].as_str().ok_or("invalid phase link")?);
    let phase = link["phase"]
        .as_str()
        .ok_or("invalid phase link")?
        .to_owned();
    if !root.exists()
        && let (Some(slug), Some(parent)) = (root.file_name().map(|s| s.to_owned()), root.parent())
    {
        // Both former layouts keep the goal folder one level below the project.
        let project = parent.parent().unwrap_or(parent);
        let direct = project.join(&slug);
        let moved = if direct.is_dir() {
            Some(direct)
        } else {
            fs::read_dir(project).ok().and_then(|entries| {
                entries
                    .flatten()
                    .map(|e| e.path().join(&slug))
                    .find(|candidate| candidate.join("plan/plan.json").is_file())
            })
        };
        if let Some(moved) = moved {
            root = moved;
            let link = serde_json::json!({"root": root, "phase": phase});
            let _ = fs::write(
                dir.join(LINK),
                serde_json::to_vec(&link).map_err(|e| e.to_string())?,
            );
        }
    }
    Ok(Some((root, phase)))
}

/// The run's phase and every phase of its goal, for the Agent Room's next step.
pub fn phase_status(dir: &Path) -> Option<serde_json::Value> {
    let (root, phase) = linked(dir).ok()??;
    let progress = crate::slug_progress::progress(&root);
    Some(serde_json::json!({
        "slug": root.file_name().map(|n| n.to_string_lossy().into_owned()),
        "phase": phase,
        "phases": progress["phases"],
        "current_phase": progress["current_phase"],
    }))
}

/// Tested candidates put the phase in verification; a merged run completes it.
pub fn sync_phase(dir: &Path) -> Result<(), String> {
    let Some((root, phase)) = linked(dir)? else {
        return Ok(());
    };
    let phase = phase.as_str();
    let state = load(dir)?.state;
    let stage = crate::slug_progress::progress(&root)["phases"]
        .as_array()
        .and_then(|p| p.iter().find(|p| p["id"] == phase))
        .and_then(|p| p["stage"].as_str().map(str::to_owned))
        .ok_or("linked phase is not in the slug plan")?;
    let tested = matches!(
        state.as_str(),
        "waiting_approval" | "approved" | "completed"
    );
    if tested && stage == "task" {
        crate::phases::advance_phase(&root, phase, "verification")?;
    }
    if state == "completed" {
        crate::phases::advance_phase(&root, phase, "completed")?;
    }
    Ok(())
}
