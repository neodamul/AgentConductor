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

/// `<repository>/<slug>` (user decision 2026-10-13: the project folder holds each goal
/// folder directly), or `<workspace>/<slug>` without a repository. A slug found in a former
/// location — `<repository>/.agentconductor/<slug>` or the interim
/// `<repository>/<workspace name>/<slug>` — is moved on first use.
pub fn root(repository: &Path, workspace: &Path, name: &str, slug: &str) -> PathBuf {
    if repository.as_os_str().is_empty() {
        return workspace.join(slug);
    }
    let root = repository.join(slug);
    if !root.exists() {
        for parent in [
            repository.join(folder_name(name, workspace)),
            repository.join(".agentconductor"),
        ] {
            if parent.join(slug).is_dir() && fs::rename(parent.join(slug), &root).is_ok() {
                mark(&root);
                // The former folder goes once nothing but its marker is left.
                let _ = fs::remove_file(parent.join(MARKER));
                if fs::remove_dir(&parent).is_err() && !parent.join(MARKER).exists() {
                    let _ = fs::write(parent.join(MARKER), MARKER_TEXT);
                }
                break;
            }
        }
    }
    root
}

/// The workspace name as a folder name: characters Windows forbids become `_`, leading dots
/// and trailing dots or spaces are dropped. An unnamed workspace uses its data folder's name.
/// Only used to find goal folders created in the interim `<repository>/<name>/` layout.
pub fn folder_name(name: &str, workspace: &Path) -> String {
    let name = if name.trim().is_empty() {
        workspace
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        name.to_owned()
    };
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "<>:\"/\\|?*".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let clean = clean
        .trim()
        .trim_start_matches('.')
        .trim_end_matches(['.', ' '])
        .trim();
    let stem = clean.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if clean.is_empty() || crate::folder::excluded(clean) {
        "workspace".into()
    } else if reserved {
        format!("{clean}_")
    } else {
        clean.into()
    }
}

/// Marks a goal folder in the project so folder copies skip it like `.git`.
pub const MARKER: &str = ".agentconductor";
const MARKER_TEXT: &str = "AgentConductor goal work tree; not project source.\n";

fn mark(folder: &Path) {
    let marker = folder.join(MARKER);
    if folder.is_dir() && !marker.exists() {
        let _ = fs::write(marker, MARKER_TEXT);
    }
}

pub fn create_tree(root: &Path) -> Result<(), String> {
    for stage in STAGES {
        fs::create_dir_all(root.join(stage)).map_err(|e| e.to_string())?;
    }
    mark(root);
    Ok(())
}

/// Keeps a goal folder (and the former `.agentconductor/`) out of Git status, candidates
/// and merges. Slug characters are `[a-z0-9_]`, so the pattern needs no escaping.
pub fn exclude_from_git(repo: &crate::git::Repository, slug: &str) -> Result<(), String> {
    validate(slug)?;
    let relative = String::from_utf8(repo.command(&["rev-parse", "--git-path", "info/exclude"])?)
        .map_err(|_| "non-UTF-8 exclude path")?;
    let path = repo.root.join(relative.trim());
    let mut text = fs::read_to_string(&path).unwrap_or_default();
    let mut changed = false;
    for (line, note) in [
        (format!("/{slug}/"), "AgentConductor goal folder"),
        (
            "/.agentconductor/".to_owned(),
            "AgentConductor slug work trees (DD-61)",
        ),
    ] {
        if !text.lines().any(|l| l.trim() == line) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&format!("# {note}\n{line}\n"));
            changed = true;
        }
    }
    if changed {
        fs::create_dir_all(path.parent().ok_or("invalid exclude path")?)
            .map_err(|e| e.to_string())?;
        fs::write(&path, text).map_err(|e| e.to_string())?;
    }
    Ok(())
}
