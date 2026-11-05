//! Reading a `/usage` screen into a quota snapshot.

use super::screens::plain;
use crate::quota::{Snapshot, snapshot, window};
use serde_json::json;

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

/// "1,351.23" -> "1351.23". Anything that is not properly grouped is left alone and rejected later.
fn thousands(amount: &str) -> String {
    let (whole, fraction) = amount.split_once('.').unwrap_or((amount, ""));
    let mut groups = whole.split(',');
    let grouped = groups.next().is_some_and(|g| (1..=3).contains(&g.len()))
        && groups.all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()));
    if !whole.contains(',') || !grouped {
        return amount.to_owned();
    }
    let digits = whole.replace(',', "");
    if fraction.is_empty() && !amount.contains('.') {
        digits
    } else {
        format!("{digits}.{fraction}")
    }
}

fn spent_budget(text: &str, at: i64) -> Result<Option<crate::quota::Budget>, String> {
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
        let used = &thousands(used.trim());
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
        let resets_at = reset.and_then(|s| crate::quota::reset_time::parse(s, at, 44640));
        return crate::quota::budget::from_spent(
            "claude",
            used,
            &thousands(limit.trim()),
            resets_at,
        )
        .map(Some);
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
                crate::quota::reset_time::parse(line, observed_at, *minutes),
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
