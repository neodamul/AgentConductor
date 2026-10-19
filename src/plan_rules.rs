//! DD-65: planning rules kept in an editable global file, overridable per workspace.
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const DEFAULT: &str = "# AgentConductor plan rules

- Start every plan with an analysis of the goal and the current codebase before dividing work.
- Divide the work into ordered phases; each phase runs task -> verification -> completed, and a phase starts only after the previous one is completed.
- Assign each task a role (verification, development, documentation, knowledge, testing) and an agent allowed for that role.
- Develop test-first (TDD): a failing test before the implementation, then make it pass.
- Keep production sources and tests in separate folders; no source or test file may exceed 300 lines.
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

/// Add repository-local Markdown rules below `.agentconductor/plan-rules/`.
/// An explicit workspace override retains its existing exclusive precedence.
pub fn for_repository(workspace: Option<&str>, repository: &Path) -> Result<String, String> {
    if let Some(text) = workspace {
        return Ok(text.to_string());
    }
    let mut rules = current(None);
    let metadata_dir = repository.join(".agentconductor");
    let metadata = match fs::symlink_metadata(&metadata_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(rules),
        Err(error) => return Err(format!("repository planning rules unavailable: {error}")),
    };
    if !metadata.file_type().is_dir() {
        return Ok(rules);
    }
    let root = metadata_dir.join("plan-rules");
    let mut files = Vec::new();
    collect_markdown(&root, &mut files)?;
    files.sort();
    for path in files {
        let relative = path
            .strip_prefix(repository)
            .map_err(|_| "planning rule escaped repository root")?;
        let text = fs::read_to_string(&path).map_err(|error| {
            format!("planning rule unreadable ({}): {error}", relative.display())
        })?;
        rules.push_str(&format!(
            "\n\nRepository planning rule ({}):\n{}\n",
            relative.display(),
            text
        ));
    }
    Ok(rules)
}

fn collect_markdown(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("planning rules directory unavailable: {error}")),
    };
    if !metadata.file_type().is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            collect_markdown(&entry.path(), files)?;
        } else if file_type.is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            files.push(entry.path());
        }
    }
    Ok(())
}

/// The rules the window shows and sends: workspace text, else the global file, else the
/// defaults — without creating the global file (it is written only when saved).
pub fn current(workspace: Option<&str>) -> String {
    if let Some(text) = workspace {
        return text.to_string();
    }
    global_path()
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_else(|| DEFAULT.to_string())
}

pub fn save_global(text: &str) -> Result<(), String> {
    let path = global_path()?;
    fs::create_dir_all(path.parent().ok_or("invalid rules path")?).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}
