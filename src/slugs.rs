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

/// R-74 / DD-92: readable folder name from an English goal — ASCII words lower-cased and
/// joined by `_`, at most 48 characters. A goal with non-English letters gets the
/// placeholder `goal_<yyyymmdd>`; the coordinator later names it in English.
pub fn suggest(goal: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    suggest_at(goal, now)
}

/// `suggest` with an explicit time for the date fallback (UTC).
pub fn suggest_at(goal: &str, epoch_secs: u64) -> String {
    if goal.chars().any(|c| c.is_alphabetic() && !c.is_ascii()) {
        return format!("goal_{}", date(epoch_secs));
    }
    let mut words = vec![String::new()];
    for c in goal.chars() {
        if c.is_ascii_alphanumeric() {
            words.last_mut().unwrap().push(c.to_ascii_lowercase());
        } else if !words.last().unwrap().is_empty() {
            words.push(String::new());
        }
    }
    let mut out = String::new();
    for word in words.into_iter().filter(|w| !w.is_empty()) {
        let extra = usize::from(!out.is_empty());
        if out.len() + extra + word.len() > MAX {
            if out.is_empty() {
                out = word[..MAX].to_string();
            }
            break;
        }
        if extra == 1 {
            out.push('_');
        }
        out.push_str(&word);
    }
    if out.is_empty() {
        format!("goal_{}", date(epoch_secs))
    } else {
        out
    }
}

/// `yyyymmdd` (UTC) from Unix seconds, using the civil-from-days algorithm.
fn date(epoch_secs: u64) -> String {
    let days = (epoch_secs / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}{month:02}{day:02}")
}

/// DD-92: `goal_<yyyymmdd>` with an optional `_<n>` suffix, awaiting an English name.
pub fn is_placeholder(slug: &str) -> bool {
    let Some(rest) = slug.strip_prefix("goal_") else {
        return false;
    };
    let (date, suffix) = rest.split_at(rest.len().min(8));
    date.len() == 8
        && date.bytes().all(|b| b.is_ascii_digit())
        && (suffix.is_empty()
            || suffix
                .strip_prefix('_')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())))
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
