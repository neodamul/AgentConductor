//! R-70 / DD-76: machine-wide interface language stored as UTF-8 JSON next to the
//! workspace registry. UI message text lives in the language-specific ID catalogs.
use std::path::PathBuf;

pub const SUPPORTED: [&str; 2] = ["en", "ko"];

fn path() -> Result<PathBuf, String> {
    Ok(crate::desktop::workspace()?
        .parent()
        .ok_or("invalid data directory")?
        .join("preferences.json"))
}

fn stored() -> serde_json::Map<String, serde_json::Value> {
    path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default()
}

/// The saved language, or English when nothing valid is saved.
pub fn load() -> String {
    stored()
        .get("language")
        .and_then(|v| v.as_str())
        .filter(|lang| SUPPORTED.contains(lang))
        .unwrap_or("en")
        .to_string()
}

/// Saves a supported language, keeping any other preferences in the file.
pub fn save(language: &str) -> Result<(), String> {
    if !SUPPORTED.contains(&language) {
        return Err(crate::ui::messages::render(
            "native.unsupported_language",
            &[("language", language)],
        ));
    }
    store("language", language)
}

/// DD-137: `light` (design E, default) or `dark` (design F).
pub const THEMES: [&str; 2] = ["light", "dark"];

pub fn load_theme() -> String {
    stored()
        .get("theme")
        .and_then(|v| v.as_str())
        .filter(|theme| THEMES.contains(theme))
        .unwrap_or("light")
        .to_string()
}

pub fn save_theme(theme: &str) -> Result<(), String> {
    if !THEMES.contains(&theme) {
        return Err(crate::ui::messages::render(
            "native.unsupported_theme",
            &[("theme", theme)],
        ));
    }
    store("theme", theme)
}

fn store(key: &str, value: &str) -> Result<(), String> {
    let path = path()?;
    let mut preferences = stored();
    preferences.insert(key.into(), value.into());
    std::fs::create_dir_all(path.parent().ok_or("invalid preferences path")?)
        .map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(&preferences).map_err(|e| e.to_string())?;
    std::fs::write(&temporary, text.as_bytes()).map_err(|e| e.to_string())?;
    std::fs::rename(&temporary, &path).map_err(|e| e.to_string())
}

/// Loads the saved language and theme for the native eframe interface.
pub fn embed(html: &str) -> String {
    html.replacen(
        "<html lang=\"en\">",
        &format!("<html lang=\"{}\" data-theme=\"{}\">", load(), load_theme()),
        1,
    )
}
