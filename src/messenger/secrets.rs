use std::path::{Path, PathBuf};

use super::Settings;

fn secrets_path() -> Result<PathBuf, String> {
    Ok(crate::desktop::workspace()?
        .parent()
        .ok_or("invalid data directory")?
        .join("messenger-secrets.json"))
}

fn key(workspace: &Path) -> String {
    let path = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    crate::workflow::digest(path.to_string_lossy().as_bytes())
}

fn secrets() -> serde_json::Map<String, serde_json::Value> {
    secrets_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

/// What the dialog may know: the settings and which secrets exist, never their values.
pub fn status(workspace: &Path, settings: &Settings) -> serde_json::Value {
    let has = |name| secret_value(workspace, name).is_some();
    serde_json::json!({"settings":settings,"has_url":has(None),"has_token":has(Some("token")),"has_relay":has(Some("relay")),"has_relay_secret":has(Some("relay_secret"))})
}

/// The workspace's webhook URL, if one is saved on this computer.
pub fn secret(workspace: &Path) -> Option<String> {
    secret_value(workspace, None)
}

fn entry(workspace: &Path, name: Option<&str>) -> String {
    match name {
        Some(name) => format!("{}#{name}", key(workspace)),
        None => key(workspace),
    }
}

/// DD-81 secrets next to the URL: `token` (Slack bot token), `relay` (relay URL), `relay_secret`.
pub fn secret_value(workspace: &Path, name: Option<&str>) -> Option<String> {
    secrets()
        .get(&entry(workspace, name))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

pub fn validate_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") || url.len() > 2048 || url.chars().any(char::is_control) {
        return Err("webhook URL must be an https:// address".into());
    }
    Ok(())
}

/// Saves (`Some`) or removes (`None`) the URL; the file is readable by the owner only.
pub fn save_secret(workspace: &Path, url: Option<&str>) -> Result<(), String> {
    save_secret_value(workspace, None, url)
}

/// Saves or removes one named secret (`None` name = webhook URL).
pub fn save_secret_value(
    workspace: &Path,
    name: Option<&str>,
    value: Option<&str>,
) -> Result<(), String> {
    let mut all = secrets();
    match value {
        Some(value) => {
            match name {
                None | Some("relay") => validate_url(value)?,
                Some(_) => {
                    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control)
                    {
                        return Err("secret must be 1–512 printable characters".into());
                    }
                }
            }
            all.insert(entry(workspace, name), value.into());
        }
        None => {
            all.remove(&entry(workspace, name));
        }
    }
    let path = secrets_path()?;
    std::fs::create_dir_all(path.parent().ok_or("invalid secrets path")?)
        .map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(&all).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    std::fs::rename(&temporary, &path).map_err(|e| e.to_string())
}
