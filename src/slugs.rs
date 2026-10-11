//! DD-61: goals become short readable slugs that own a work tree.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const STAGES: [&str; 5] = ["plan", "phase", "task", "verification", "completed"];
const MAX: usize = 48;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slug {
    pub slug: String,
    pub goal: String,
    pub created_ms: u64,
}

/// Lowercase ASCII words joined by `_`; non-ASCII goals get the editable placeholder `goal`.
pub fn suggest(goal: &str) -> String {
    let mut out = String::new();
    for word in goal
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
    {
        let word = word.to_ascii_lowercase();
        let extra = usize::from(!out.is_empty());
        if out.len() + extra + word.len() > MAX {
            break;
        }
        if extra == 1 {
            out.push('_');
        }
        out.push_str(&word);
    }
    if out.is_empty() { "goal".into() } else { out }
}

pub fn unique(base: &str, existing: &[String]) -> String {
    if !existing.iter().any(|s| s == base) {
        return base.into();
    }
    (2..)
        .map(|n| {
            let suffix = format!("_{n}");
            format!("{}{suffix}", &base[..base.len().min(MAX - suffix.len())])
        })
        .find(|candidate| !existing.contains(candidate))
        .expect("an unused suffix exists")
}

pub fn validate(slug: &str) -> Result<(), String> {
    let ok = !slug.is_empty()
        && slug.len() <= MAX
        && slug.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if ok {
        Ok(())
    } else {
        Err(
            "slug must be 1-48 characters of a-z, 0-9 and _, starting with a letter or digit"
                .into(),
        )
    }
}

/// `<repository>/.agentconductor/<slug>` or `<workspace>/<slug>` without a repository.
pub fn root(repository: &Path, workspace: &Path, slug: &str) -> PathBuf {
    if repository.as_os_str().is_empty() {
        workspace.join(slug)
    } else {
        repository.join(".agentconductor").join(slug)
    }
}

pub fn create_tree(root: &Path) -> Result<(), String> {
    for stage in STAGES {
        fs::create_dir_all(root.join(stage)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Keeps the repository's `.agentconductor/` out of Git status, candidates and merges.
pub fn exclude_from_git(repo: &crate::git::Repository) -> Result<(), String> {
    let relative = String::from_utf8(repo.command(&["rev-parse", "--git-path", "info/exclude"])?)
        .map_err(|_| "non-UTF-8 exclude path")?;
    let path = repo.root.join(relative.trim());
    let existing = fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == "/.agentconductor/") {
        return Ok(());
    }
    fs::create_dir_all(path.parent().ok_or("invalid exclude path")?).map_err(|e| e.to_string())?;
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("# AgentConductor slug work trees (DD-61)\n/.agentconductor/\n");
    fs::write(&path, text).map_err(|e| e.to_string())
}
