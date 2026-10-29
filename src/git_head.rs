//! GitHub #234 (DD-203): the repository HEAD for the state revision, without a Git process on the request path.
//! `read` takes it from the files Git itself reads (`HEAD`, the loose ref, `packed-refs`). A layout those files cannot
//! answer (a `.git` file, a reftable, an unborn branch) is asked of Git by a background task, never by a request:
//! `Heads::get` returns the last known value at once and refreshes it off the request path only when it is stale.
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// A fallback answer older than this is asked of Git again, in the background, the next time it is wanted.
const STALE: Duration = Duration::from_secs(5);

fn is_commit(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A ref name that stays inside `refs/`; anything else is not trusted to be joined to a path.
pub fn resolve_ref_name(name: &str) -> Option<&str> {
    let ok = name.starts_with("refs/")
        && !name.contains(['\\', ':', '\0'])
        && name
            .split('/')
            .all(|part| !part.is_empty() && part != ".." && part != "." && !part.starts_with('.'));
    ok.then_some(name)
}

/// The Git directory of a project: its own `.git` directory, or the history kept outside a project that has none.
fn git_dir(root: &Path) -> Option<PathBuf> {
    let own = root.join(".git");
    if fs::symlink_metadata(&own).ok()?.is_dir() {
        return Some(own);
    }
    crate::folder::external_metadata(root).ok().flatten()
}

/// `git rev-parse HEAD` answered from files, or `None` when they cannot answer it.
pub fn read(project: &Path) -> Option<String> {
    let dir = git_dir(&project.canonicalize().ok()?)?;
    let head = fs::read_to_string(dir.join("HEAD")).ok()?;
    let head = head.trim();
    if is_commit(head) {
        return Some(head.into());
    }
    let name = resolve_ref_name(head.strip_prefix("ref: ")?.trim())?;
    if let Ok(loose) = fs::read_to_string(dir.join(name))
        && is_commit(loose.trim())
    {
        return Some(loose.trim().into());
    }
    let packed = fs::read_to_string(dir.join("packed-refs")).ok()?;
    packed.lines().find_map(|line| {
        let (commit, reference) = line.split_once(' ')?;
        (reference == name && is_commit(commit)).then(|| commit.into())
    })
}

/// The HEAD for code that may wait for it: from the files, else from Git.
pub fn head(project: &Path) -> Result<String, String> {
    match read(project) {
        Some(commit) => Ok(commit),
        None => crate::git::Repository::open(project)?.head(),
    }
}

/// What a request learns about HEAD.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Head {
    Known(String),
    /// Git has not answered yet (the first request for this folder); the next request will know.
    Pending,
    Failed,
}

impl Head {
    pub fn bytes(&self) -> Vec<u8> {
        match self {
            Head::Known(commit) => commit.as_bytes().to_vec(),
            Head::Pending => b"head-pending".to_vec(),
            Head::Failed => b"head-unavailable".to_vec(),
        }
    }
}

struct Entry {
    head: Head,
    at: Instant,
    refreshing: bool,
}

/// Per-project fallback answers, refreshed by short background tasks (at most one per folder at a time).
#[derive(Clone, Default)]
pub struct Heads {
    entries: Arc<Mutex<HashMap<PathBuf, Entry>>>,
}

impl Heads {
    /// Never waits for a process: a file read, a lock held for a lookup, and at most one spawned refresh.
    pub fn get(&self, project: &Path) -> Head {
        if let Some(commit) = read(project) {
            return Head::Known(commit);
        }
        let Ok(mut entries) = self.entries.lock() else {
            return Head::Failed;
        };
        let entry = entries.entry(project.to_path_buf()).or_insert(Entry {
            head: Head::Pending,
            at: Instant::now(),
            refreshing: false,
        });
        let answer = entry.head.clone();
        if !entry.refreshing && (answer == Head::Pending || entry.at.elapsed() >= STALE) {
            entry.refreshing = true;
            let (shared, path) = (self.entries.clone(), project.to_path_buf());
            let started = std::thread::Builder::new()
                .name("head-refresh".into())
                .spawn(move || {
                    let head = head(&path).map_or(Head::Failed, Head::Known);
                    if let Ok(mut entries) = shared.lock()
                        && let Some(entry) = entries.get_mut(&path)
                    {
                        *entry = Entry {
                            head,
                            at: Instant::now(),
                            refreshing: false,
                        };
                    }
                });
            if started.is_err() {
                entry.refreshing = false;
            }
        }
        answer
    }
}
