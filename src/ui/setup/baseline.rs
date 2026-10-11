//! DD-50: explain an unclean repository and commit it as the baseline only on request.
use super::*;

const MAX_CHANGES: usize = 200;

impl Startup {
    pub(super) fn baseline_status(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        let repo = Repository::open(&saved.config.repository)?;
        let status = repo.command(&["status", "--porcelain", "--untracked-files=all"])?;
        let lines: Vec<String> = String::from_utf8_lossy(&status)
            .lines()
            .map(str::to_owned)
            .collect();
        let total = lines.len();
        Ok(serde_json::json!({
            "clean": total == 0,
            "changes": lines.into_iter().take(MAX_CHANGES).collect::<Vec<_>>(),
            "total": total,
        }))
    }

    /// Commits every current change with the operator's own Git identity and hooks.
    pub(super) fn commit_baseline(&self, saved: &Saved) -> Result<serde_json::Value, String> {
        let repo = Repository::open(&saved.config.repository)?;
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
