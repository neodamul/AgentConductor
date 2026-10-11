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

/// Tested candidates put the phase in verification; a merged run completes it.
pub fn sync_phase(dir: &Path) -> Result<(), String> {
    let Ok(bytes) = fs::read(dir.join(LINK)) else {
        return Ok(());
    };
    let link: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let root = PathBuf::from(link["root"].as_str().ok_or("invalid phase link")?);
    let phase = link["phase"].as_str().ok_or("invalid phase link")?;
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
