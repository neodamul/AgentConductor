//! Path spellings and tool-input shapes the policy accepts.
use serde_json::Value;

pub(super) fn safe_relative(path: &str) -> Result<(), String> {
    if path.len() > 4096
        || path.contains(['\\', '\0', '\n', '\r'])
        || path.split('/').count() > 64
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("tool path must be an unambiguous bounded workspace-relative path".into());
    }
    if metadata(path) {
        return Err(
            "repository metadata and agent configuration are outside the tool policy".into(),
        );
    }
    Ok(())
}

/// Repository metadata and agent configuration, in any case and either separator.
pub(super) fn metadata(path: &str) -> bool {
    path.split(['/', '\\']).any(|part| {
        matches!(
            part.to_ascii_lowercase().as_str(),
            ".git" | ".claude" | ".codex" | ".agents" | ".agentconductor" | ".mcp.json"
        )
    })
}

pub(super) fn sensitive(path: &str) -> bool {
    path.split('/').any(|part| {
        let part = part.to_ascii_lowercase();
        matches!(
            part.as_str(),
            "secrets" | "secret" | ".aws" | ".ssh" | ".gnupg" | ".npmrc" | ".netrc" | ".pypirc"
        ) || part == ".env"
            || part.starts_with(".env.")
            || part.ends_with(".pem")
            || part.ends_with(".key")
            || part.starts_with("credentials")
            || part.starts_with("secrets.")
            || part.starts_with("id_rsa")
            || part.starts_with("id_ed25519")
    })
}

pub(super) fn string<'a>(input: &'a Value, key: &str, nonempty: bool) -> Result<&'a str, String> {
    input[key]
        .as_str()
        .filter(|s| !nonempty || !s.trim().is_empty())
        .ok_or_else(|| format!("tool input requires a valid {key}"))
}

pub(super) fn keys(input: &Value, allowed: &[&str]) -> Result<(), String> {
    let fields = input.as_object().ok_or("tool input must be an object")?;
    if fields.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("unrecognized tool input field; execution refused".into());
    }
    Ok(())
}
