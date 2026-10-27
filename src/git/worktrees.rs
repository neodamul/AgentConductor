use super::Repository;
use std::path::{Path, PathBuf};

impl Repository {
    /// Removes this repository's linked worktrees that live under `run` and prunes entries whose
    /// folder is already gone; other worktrees are never touched. Returns how many entries went.
    pub fn remove_worktrees_under(&self, run: &Path) -> Result<usize, String> {
        let run = run.canonicalize().map_err(|e| e.to_string())?;
        let listing = self.text(&["worktree", "list", "--porcelain"])?;
        let mut removed = 0;
        for line in listing.lines() {
            let Some(path) = line.strip_prefix("worktree ").map(PathBuf::from) else {
                continue;
            };
            // A vanished folder cannot be canonicalized; its parent still shows where it lived.
            let inside = path
                .canonicalize()
                .or_else(|_| {
                    path.parent()
                        .ok_or(std::io::ErrorKind::NotFound.into())
                        .and_then(|p| p.canonicalize())
                        .map(|p| p.join(path.file_name().unwrap_or_default()))
                })
                .is_ok_and(|p| p != self.root && p.starts_with(&run));
            if inside {
                let target = path.to_str().ok_or("non-UTF8 worktree path")?;
                if self
                    .command(&["worktree", "remove", "--force", target])
                    .is_ok()
                {
                    removed += 1;
                }
            }
        }
        let before = self
            .text(&["worktree", "list", "--porcelain"])?
            .lines()
            .count();
        self.command(&["worktree", "prune"])?;
        let after = self
            .text(&["worktree", "list", "--porcelain"])?
            .lines()
            .count();
        // `prune` forgets entries whose folder vanished; each entry is three lines.
        Ok(removed + before.saturating_sub(after).div_ceil(3))
    }
}
