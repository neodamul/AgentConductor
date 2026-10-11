//! DD-65: planning rules kept in an editable global file, overridable per workspace.
use std::{fs, path::PathBuf};

pub const DEFAULT: &str = "# AgentConductor plan rules

- Start every plan with an analysis of the goal and the current codebase before dividing work.
- Divide the work into ordered phases; each phase runs task -> verification -> completed, and a phase starts only after the previous one is completed.
- Assign each task a role (verification, development, documentation, knowledge, testing) and an agent allowed for that role.
- Develop test-first (TDD): a failing test before the implementation, then make it pass.
- Keep production sources and tests in separate folders; no source or test file may exceed 500 lines.
- Write all documents in English; trace requirements -> business goals -> system features -> use cases -> functional/non-functional analysis -> QAS -> design drivers -> design decisions.
- Record completed work units (commit/push, collaboration memory) and turn verified knowledge into wiki assets.
- Final approval and merge stay with the human operator.
";

/// `~/.agentconductor/plan-rules.md` (Windows: `%USERPROFILE%`).
pub fn global_path() -> Result<PathBuf, String> {
    let workspace = crate::desktop::workspace()?;
    Ok(workspace
        .parent()
        .ok_or("invalid data directory")?
        .join("plan-rules.md"))
}

/// The workspace override when set, otherwise the global file (created with defaults once).
pub fn effective(workspace: Option<&str>) -> Result<String, String> {
    if let Some(text) = workspace {
        return Ok(text.to_string());
    }
    let path = global_path()?;
    if !path.exists() {
        fs::create_dir_all(path.parent().ok_or("invalid rules path")?)
            .map_err(|e| e.to_string())?;
        fs::write(&path, DEFAULT).map_err(|e| e.to_string())?;
    }
    fs::read_to_string(&path).map_err(|e| format!("plan rules unreadable: {e}"))
}
