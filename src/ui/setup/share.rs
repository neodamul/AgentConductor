//! DD-59: workspace settings as a Git-friendly file, without secrets or machine state.
use super::*;

mod export;
mod import;
mod preview;

const FILE: &str = "agentconductor.workspace.json";
const FORMAT: &str = "agentconductor.workspace/v1";
/// Capabilities asserted by an operator about their own machine (DD-17).
const LOCAL_CLAIMS: [&str; 2] = ["external_sandbox", "read_only"];

fn folder(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(crate::ui::messages::text("native.choose_an_absolute_folder").into());
    }
    path.canonicalize().map_err(|e| {
        crate::ui::messages::render("native.folder_unavailable", &[("e", &e.to_string())])
    })
}

fn read_shared(dir: &Path) -> Result<serde_json::Value, String> {
    let path = folder(dir)?.join(FILE);
    let text = fs::read_to_string(&path)
        .map_err(|_| crate::ui::messages::render("native.no_in_that_folder", &[("FILE", FILE)]))?;
    if text.len() > 256 * 1024 {
        return Err(
            crate::ui::messages::text("native.workspace_settings_file_exceeds_256_kib").into(),
        );
    }
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        crate::ui::messages::render("native.invalid", &[("FILE", FILE), ("e", &e.to_string())])
    })?;
    if value["format"] != FORMAT {
        return Err(crate::ui::messages::render(
            "native.unsupported_settings_format_expected",
            &[("FORMAT", FORMAT)],
        ));
    }
    Ok(value)
}
