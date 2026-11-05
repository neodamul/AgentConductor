//! DD-229: at most ten non-actionable notices a minute per workspace; the rest are counted and
//! reported once as "N more events". Notices that carry a code are never counted or folded.
use serde::{Deserialize, Serialize};
use std::path::Path;

const LIMIT: usize = 10;
const WINDOW: u64 = 60;

#[derive(Default, Serialize, Deserialize)]
struct Rate {
    sent: Vec<u64>,
    folded: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Admit {
    Send,
    /// Send a summary of this many folded notices first.
    SendAfterSummary(u32),
    Fold,
}

fn path(workspace: &Path) -> std::path::PathBuf {
    workspace.join("messenger-rate.json")
}

/// Decides one notice. Best effort: an unreadable or unwritable file never blocks a notice.
pub fn admit(workspace: &Path, now: u64, actionable: bool) -> Admit {
    if actionable {
        return Admit::Send;
    }
    let mut rate: Rate = std::fs::read_to_string(path(workspace))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    rate.sent.retain(|t| *t + WINDOW > now);
    let verdict = if rate.sent.len() < LIMIT {
        rate.sent.push(now);
        match std::mem::take(&mut rate.folded) {
            0 => Admit::Send,
            n => Admit::SendAfterSummary(n),
        }
    } else {
        rate.folded = rate.folded.saturating_add(1);
        Admit::Fold
    };
    if let Ok(text) = serde_json::to_vec(&rate) {
        let temp = path(workspace).with_extension("tmp");
        if std::fs::write(&temp, text).is_ok() {
            let _ = std::fs::rename(temp, path(workspace));
        }
    }
    verdict
}

#[cfg(test)]
#[path = "../../tests/unit/messenger_throttle.rs"]
mod tests;
