//! #130: the repository releases come from, kept in `preferences.json` next to the interface
//! language. A github.com repository is `owner/name`; any other GitHub-compatible server (for
//! example an in-house GitHub Enterprise) is `https://host/owner/name`. Empty means the default.
use serde_json::{Value, json};

pub const DEFAULT_REPOSITORY: &str = "neodamul/AgentConductor";

fn saved() -> String {
    crate::ui::language::stored_text("update_repository").unwrap_or_default()
}

pub fn repository() -> String {
    Some(saved())
        .filter(|saved| !saved.is_empty())
        .unwrap_or_else(|| DEFAULT_REPOSITORY.into())
}

/// The latest-release API of the saved repository; release assets come from that release.
pub fn api_url() -> String {
    let repository = repository();
    match repository
        .strip_prefix("https://")
        .map(|r| r.split_once('/'))
    {
        None => format!("https://api.github.com/repos/{repository}/releases/latest"),
        Some(Some((host, path))) => format!("https://{host}/api/v3/repos/{path}/releases/latest"),
        // Unreachable for saved values: normalize_repository always keeps a path.
        Some(None) => format!("{repository}/releases/latest"),
    }
}

/// `owner/name` or `https://host/owner/name[.git]`; github.com URLs shrink to `owner/name`.
pub fn normalize_repository(input: &str) -> Result<String, String> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(String::new());
    }
    let invalid =
        || crate::ui::messages::render("native.update_repository_invalid", &[("value", input)]);
    let (host, path) = match input.strip_prefix("https://") {
        Some(rest) => {
            let (host, path) = rest.split_once('/').ok_or_else(invalid)?;
            (Some(host), path)
        }
        None if input.contains("://") => return Err(invalid()),
        None => (None, input),
    };
    let path = path.trim_end_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let part = |part: &str, dots: bool| {
        !part.is_empty()
            && part.len() <= 100
            && !part.starts_with('-')
            && !matches!(part, "." | "..")
            && part
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || (dots && matches!(c, '.' | '_')))
    };
    let host_ok = |host: &str| {
        !host.starts_with(['.', '-', ':'])
            && host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
    };
    match (host, path.split_once('/')) {
        (_, Some((owner, name))) if part(owner, false) && part(name, true) => match host {
            None | Some("github.com") => Ok(path.into()),
            Some(host) if host_ok(host) => Ok(format!("https://{host}/{path}")),
            Some(_) => Err(invalid()),
        },
        _ => Err(invalid()),
    }
}

/// Validates first so a bad value saves nothing.
pub fn save(repository: &str) -> Result<(), String> {
    crate::ui::language::store("update_repository", &normalize_repository(repository)?)
}

pub fn settings() -> Value {
    json!({"repository": saved(), "default_repository": DEFAULT_REPOSITORY})
}
