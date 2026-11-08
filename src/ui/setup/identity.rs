//! GitHub #562: the footer names the Workspace folder and its Git branch, read from `.git/HEAD` without running git.
use serde_json::{Value, json};
use std::path::Path;

/// `null` when no folder is chosen; `branch` is `null` outside a Git checkout, a short commit when detached.
pub fn repository_identity(repository: &Path) -> Value {
    let Some(name) = repository.file_name().and_then(|n| n.to_str()) else {
        return Value::Null;
    };
    json!({"path":repository,"name":name,"branch":branch(repository)})
}

fn branch(repository: &Path) -> Option<String> {
    let dot = repository.join(".git");
    let git_dir = match std::fs::read_to_string(&dot) {
        // A linked worktree keeps a `gitdir: <path>` pointer file instead of a directory.
        Ok(pointer) => repository.join(pointer.strip_prefix("gitdir:")?.trim()),
        Err(_) => dot,
    };
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    match head.strip_prefix("ref: refs/heads/") {
        Some(name) => Some(name.to_string()),
        None => head.get(..7).map(str::to_string),
    }
}
