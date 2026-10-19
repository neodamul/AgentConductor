use super::{
    Snapshot,
    parser::{snapshot, window},
};
use serde_json::json;

fn plain(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 27 {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            if bytes[i] == b'[' {
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
            } else if bytes[i] == b']' {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 7 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == 27 && bytes.get(i + 1) == Some(&b'\\') {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            } else {
                while i < bytes.len() && (0x20..=0x2f).contains(&bytes[i]) {
                    i += 1;
                }
                i += 1;
            }
        } else {
            if bytes[i] >= 32 || bytes[i] == b'\n' {
                out.push(bytes[i]);
            }
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Claude's folder-trust screen. The usage session runs with no tools, hooks or setting
/// sources, so it is accepted with its default "Yes, proceed" (user, 2026-10-13).
pub fn claude_trust_prompt(text: &str) -> bool {
    text.contains("Permission Required:")
        || text.contains("Do you trust the files")
        || text.contains("Yes, I trust this folder")
        || yes_no(text)
}

/// Screen-reader mode asks "Enter y/n:" and repeats it until y or n is typed.
fn yes_no(text: &str) -> bool {
    text.contains("Enter y/n") || text.contains("Please answer y or n")
}

/// The keys that accept the trust screen: Enter takes the default "Yes, proceed", and the
/// screen-reader form needs an explicit y.
pub fn claude_trust_answer(text: &str) -> &'static [u8] {
    if yes_no(text) { b"y\r" } else { b"\r" }
}

/// The trust screen is answered at most this many times before the query gives up.
pub(crate) const TRUST_ANSWERS: u8 = 2;
pub(crate) const TRUST_REFUSED: &str =
    "Claude kept asking to trust the project folder; open claude there once and accept it";

/// DD-94: screens that need the operator; the collector does not answer them.
pub fn claude_blocker(text: &str) -> Option<&'static str> {
    if text.contains("Let's get started.") || text.contains("Choose the text style") {
        return Some(
            "Claude CLI first-run setup is not finished on this computer; run claude once in a terminal, complete setup and sign in, then refresh",
        );
    }
    None
}

/// "Current week (all models)" is the shared window; a named model gets its own label so
/// a later per-model line (e.g. "Current week (Fable)") never overwrites the shared one.
fn week_label(line: &str) -> String {
    let model = line
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'))
        .map(|(name, _)| name.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let slug: String = model
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(32)
        .collect();
    let slug = slug.trim_matches('_');
    if slug.is_empty() || model == "all models" {
        "seven_day".into()
    } else if model.contains("opus") {
        "seven_day_opus".into()
    } else if model.contains("sonnet") {
        "seven_day_sonnet".into()
    } else {
        format!("seven_day_{slug}")
    }
}

/// Version-sensitive screen-reader metadata; unrelated statistics never count as capacity.
pub fn claude_usage_ready(text: &str) -> bool {
    !claude_trust_prompt(text)
        && claude_blocker(text).is_none()
        && text.contains("Claude Code v")
        && text.lines().any(|line| {
            let line = line.trim();
            ["$", ">", "❯"].iter().any(|prompt| {
                line.strip_prefix(prompt)
                    .is_some_and(|tail| tail.is_empty() || tail.starts_with(' '))
            })
        })
}
fn quota_timeout(sent: bool) -> &'static str {
    if sent {
        "Claude quota response timed out after /usage; no generation requested"
    } else {
        "Claude quota prompt timed out before /usage; no generation requested"
    }
}
fn spent_budget(text: &str, at: i64) -> Result<Option<super::Budget>, String> {
    let lines: Vec<_> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if !line.contains("spent") {
            continue;
        }
        let Some((_, amount)) = line.split_once('$') else {
            continue;
        };
        let Some((used, rest)) = amount.split_once('$') else {
            continue;
        };
        let Some(used) = used.trim().strip_suffix('/') else {
            continue;
        };
        let Some((limit, tail)) = rest.split_once(char::is_whitespace) else {
            continue;
        };
        if !tail.trim_start().starts_with("spent") {
            continue;
        }
        let reset = tail.find("Resets ").map(|i| &tail[i..]).or_else(|| {
            lines
                .get(index + 1)
                .copied()
                .map(str::trim)
                .filter(|s| s.starts_with("Resets "))
        });
        let resets_at = reset.and_then(|s| super::reset_time::parse(s, at, 44640));
        return super::budget::from_spent("claude", used.trim(), limit.trim(), resets_at).map(Some);
    }
    Ok(None)
}

pub fn parse_claude_usage(text: &str, observed_at: i64) -> Result<Snapshot, String> {
    let text = plain(text);
    if text.contains("Permission Required:") {
        return Err("Claude project trust approval is required; no approval was sent".into());
    }
    let enterprise = text.lines().take(12).any(|line| {
        let line = line.to_ascii_lowercase();
        line.contains("enterprise plan") || line.contains("claude enterprise")
    });
    let budget = spent_budget(&text, observed_at)?;
    let mut pending: Option<(String, u64)> = None;
    let mut active: Option<(String, u64)> = None;
    let mut resets: std::collections::BTreeMap<String, Option<i64>> =
        std::collections::BTreeMap::new();
    let mut windows = std::collections::BTreeMap::new();
    for line in text.lines().map(str::trim) {
        if line.contains("spent") && line.contains("$") && line.contains("/") {
            pending = None;
            active = None;
            continue;
        }
        if line == "Current session" {
            pending = Some(("five_hour".into(), 300));
            active = pending.clone();
            continue;
        }
        if line.starts_with("Current week") {
            pending = Some((week_label(line), 10080));
            active = pending.clone();
            continue;
        }
        if line.starts_with("What's contributing") || line == "Usage credits" || line == "Session" {
            pending = None;
            active = None;
        }
        if let Some((label, minutes)) = &active
            && line.starts_with("Resets ")
        {
            resets.insert(
                label.clone(),
                super::reset_time::parse(line, observed_at, *minutes),
            );
        }
        if let Some((label, minutes)) = &pending
            && line.contains("used")
            && let Some((prefix, _)) = line.split_once('%')
        {
            let used = prefix
                .trim()
                .parse::<f64>()
                .map_err(|_| "unrecognized Claude quota percentage")?;
            if !used.is_finite() || !(0.0..=100.0).contains(&used) {
                return Err("invalid Claude quota percentage".into());
            }
            windows.insert(
                label.clone(),
                window(
                    "claude",
                    label,
                    &json!({"used_percentage":used}),
                    true,
                    Some(*minutes),
                )?,
            );
            pending = None;
        }
    }
    if windows.is_empty() && budget.is_none() {
        return Err("Claude /usage did not report recognized capacity windows".into());
    }
    let mut result = snapshot(
        "claude /usage screen-reader metadata",
        observed_at,
        windows
            .into_values()
            .map(|mut w| {
                w.resets_at = resets.get(&w.label).copied().flatten();
                w
            })
            .collect(),
    )?;
    result.budgets = budget.into_iter().collect();
    if enterprise {
        result.plan_type = Some("enterprise".into());
    }
    Ok(result)
}

/// Claude only answers /usage inside a project folder it already trusts.
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
fn require_project(cwd: &std::path::Path) -> Result<(), String> {
    if cwd.as_os_str().is_empty() {
        return Err(
            "Choose a project folder for this workspace first; Claude reports usage only inside a trusted project folder"
                .into(),
        );
    }
    Ok(())
}

#[cfg(not(windows))]
mod query;
#[cfg(not(windows))]
pub use query::query_claude;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::query_claude;
