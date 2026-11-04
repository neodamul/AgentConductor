//! GitHub #468: the About screen is shown once per installed version (first start after an install or an update) and
//! stays reachable from the menu. The last version it was shown for is an app preference; no polling, nothing sent.
use super::language;

const KEY: &str = "about_seen_version";

/// `show` when this version was never shown; `updated` when an older version was shown before (an update, not a fresh install).
pub(super) fn status() -> serde_json::Value {
    let version = crate::update::current();
    let seen = language::stored_text(KEY);
    serde_json::json!({
        "version": version,
        "show": seen.as_deref() != Some(version.as_str()),
        "updated": seen.as_deref().is_some_and(|old| old != version),
    })
}

/// Remembers that this version's About screen was shown.
pub(super) fn seen() -> Result<serde_json::Value, String> {
    language::store(KEY, &crate::update::current())?;
    Ok(status())
}
