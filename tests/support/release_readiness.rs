//! #420: non-publishing release-readiness evidence with offline fixture validation.
use std::path::Path;
pub const VERSION: &str = "0.2.0";
#[derive(Clone)]
pub struct Evidence {
    pub cargo: String,
    pub documents: Vec<(String, Option<String>)>,
    pub release_notes: bool,
    pub open_issues: Vec<u64>,
    pub screenshots: Result<(), String>,
}
pub fn failures(e: &Evidence) -> Vec<String> {
    let mut reasons = Vec::new();
    if e.cargo != VERSION {
        reasons.push(format!("Cargo version {} must be {VERSION}", e.cargo));
    }
    for (name, version) in &e.documents {
        if version.as_deref() != Some(VERSION) {
            reasons.push(format!("{name} version {version:?} must be {VERSION}"));
        }
    }
    if !e.release_notes {
        reasons.push(format!(
            "Missing versioned release notes for {VERSION}; Unreleased is insufficient"
        ));
    }
    if !e.open_issues.is_empty() {
        reasons.push(format!(
            "Open milestone issues: {}",
            e.open_issues
                .iter()
                .map(|n| format!("#{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if let Err(reason) = &e.screenshots {
        reasons.push(format!("Screenshot freshness: {reason}"));
    }
    reasons
}
pub fn marked_version(text: &str, open: &str, close: &str) -> Option<String> {
    let value = text.split_once(open)?.1.split_once(close)?.0.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
pub fn has_release_notes(text: &str) -> bool {
    let mut inside = false;
    for line in text.lines() {
        if let Some(header) = line.strip_prefix("## ") {
            inside = header
                .split_whitespace()
                .next()
                .map(|s| s.trim_matches(['[', ']']).trim_start_matches('v'))
                == Some(VERSION);
        } else if inside && line.trim_start().starts_with("- ") {
            return true;
        }
    }
    false
}
pub fn collect(
    root: &Path,
    cargo: &str,
    open_issues: Vec<u64>,
    screenshots: Result<(), String>,
) -> Evidence {
    let documents = [
        ("README.md", "**Version:** ", " ·"),
        ("docs/user-manual.md", "Version **", "**"),
        ("docs/user-manual.ko.md", "버전 **", "**"),
        ("docs/user-manual.html", "<p>Version <strong>", "</strong>"),
        ("docs/user-manual.ko.html", "<p>버전 <strong>", "</strong>"),
    ]
    .into_iter()
    .map(|(name, open, close)| {
        let value = std::fs::read_to_string(root.join(name))
            .ok()
            .and_then(|text| marked_version(&text, open, close));
        (name.to_owned(), value)
    })
    .collect();
    Evidence {
        cargo: cargo.into(),
        documents,
        open_issues,
        screenshots,
        release_notes: std::fs::read_to_string(root.join("CHANGELOG.md"))
            .is_ok_and(|text| has_release_notes(&text)),
    }
}
