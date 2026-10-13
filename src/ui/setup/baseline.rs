//! DD-50: explain an unclean repository and commit it as the baseline only on request.
use super::*;

const MAX_CHANGES: usize = 200;

impl Startup {
    pub(super) fn baseline_status(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        if let Some(needed) = project::setup_needed(&saved.config.repository)? {
            return Ok(
                serde_json::json!({"needs_git":needed=="git","needs_initial_commit":needed=="commit","repository":saved.config.repository,"source_mode":"folder","clean":true,"changes":[],"os_files":[],"total":0}),
            );
        }
        let repo = project::repository(&saved.config.repository)?;
        let status = repo.command(&["status", "--porcelain", "--untracked-files=all"])?;
        let lines: Vec<String> = String::from_utf8_lossy(&status)
            .lines()
            .map(str::to_owned)
            .collect();
        let total = lines.len();
        // DD-52: only untracked files can be hidden by an exclude pattern.
        let os_files: Vec<&str> = lines
            .iter()
            .filter_map(|line| line.strip_prefix("?? "))
            .filter(|path| os_pattern(path).is_some())
            .collect();
        Ok(serde_json::json!({
            "clean": total == 0,
            "os_files": os_files,
            "changes": lines.iter().take(MAX_CHANGES).collect::<Vec<_>>(),
            "total": total,
        }))
    }

    /// Adds missing OS-metadata patterns to the repository's local, untracked exclude file.
    pub(super) fn ignore_os_files(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        let status = self.baseline_status(saved)?;
        let mut patterns: Vec<&str> = status["os_files"]
            .as_array()
            .ok_or(crate::ui::messages::text("native.invalid_baseline_status"))?
            .iter()
            .filter_map(|path| path.as_str().and_then(os_pattern))
            .collect();
        patterns.sort_unstable();
        patterns.dedup();
        let repo = project::repository(&saved.config.repository)?;
        let relative =
            String::from_utf8(repo.command(&["rev-parse", "--git-path", "info/exclude"])?)
                .map_err(|_| crate::ui::messages::text("native.non_utf_8_exclude_path"))?;
        let path = repo.root.join(relative.trim());
        let existing = fs::read_to_string(&path).unwrap_or_default();
        let missing: Vec<&str> = patterns
            .into_iter()
            .filter(|pattern| !existing.lines().any(|line| line.trim() == *pattern))
            .collect();
        if !missing.is_empty() {
            fs::create_dir_all(
                path.parent()
                    .ok_or(crate::ui::messages::text("native.invalid_exclude_path"))?,
            )
            .map_err(|e| e.to_string())?;
            let mut text = existing;
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str("# Operating-system metadata ignored by AgentConductor\n");
            for pattern in &missing {
                text.push_str(pattern);
                text.push('\n');
            }
            fs::write(&path, text).map_err(|e| e.to_string())?;
        }
        Ok(serde_json::json!({"ignored":missing,"status":self.baseline_status(saved)?}))
    }

    /// Commits every current change with the operator's own Git identity and hooks.
    pub(super) fn commit_baseline(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        let repo = project::repository(&saved.config.repository)?;
        if repo.clean().is_ok() {
            return Ok(serde_json::json!({"committed":false,"head":repo.head()?}));
        }
        let summary: String = saved
            .config
            .goal
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("work in progress")
            .chars()
            .take(72)
            .collect();
        repo.command(&["add", "-A"])?;
        repo.command(&[
            "commit",
            "-m",
            &format!("AgentConductor baseline: {summary}"),
        ])?;
        repo.clean()?;
        Ok(serde_json::json!({"committed":true,"head":repo.head()?}))
    }
}

/// The exclude pattern for an operating-system metadata file, if `path` is one.
fn os_pattern(path: &str) -> Option<&'static str> {
    let name = path.trim_end_matches('/').rsplit('/').next()?;
    match name {
        ".DS_Store" => Some(".DS_Store"),
        "Thumbs.db" => Some("Thumbs.db"),
        "ehthumbs.db" => Some("ehthumbs.db"),
        "desktop.ini" => Some("desktop.ini"),
        _ if name.starts_with("._") => Some("._*"),
        _ => None,
    }
}
