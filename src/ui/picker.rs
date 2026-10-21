//! DD-46: native folder selection through tools the operating system ships. The result
//! only fills a form field; saving still validates the path.
#[cfg(target_os = "macos")]
#[path = "picker/macos.rs"]
mod macos;
#[cfg(windows)]
#[path = "picker/windows.rs"]
pub(crate) mod windows;
use std::path::PathBuf;
#[cfg(not(any(target_os = "macos", windows)))]
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FolderPurpose {
    Project,
    WorkspaceSettingsExport,
    WorkspaceSettingsImport,
}

pub fn folder_picker_title(purpose: FolderPurpose, language: &str) -> &'static str {
    let key = match purpose {
        FolderPurpose::Project => "native.choose_a_project_folder",
        FolderPurpose::WorkspaceSettingsExport => "native.choose_folder_for_settings_export",
        FolderPurpose::WorkspaceSettingsImport => "native.choose_folder_for_settings_import",
    };
    crate::ui::messages::in_language(language, key)
}

pub fn pick_folder(purpose: FolderPurpose) -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_FOLDER") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    let title = folder_picker_title(purpose, &crate::ui::language::load());
    #[cfg(target_os = "macos")]
    return macos::pick(true, title);
    #[cfg(not(any(target_os = "macos", windows)))]
    return run(dialog());
    #[cfg(windows)]
    return windows::pick(true, false, title);
}

/// DD-60: choose a Markdown plan document; the result is only a path to read.
pub fn pick_file() -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_FILE") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    #[cfg(target_os = "macos")]
    return macos::pick(
        false,
        crate::ui::messages::text("native.choose_a_plan_document"),
    );
    #[cfg(not(any(target_os = "macos", windows)))]
    return run(file_dialog(false));
    #[cfg(windows)]
    return windows::pick(
        false,
        false,
        crate::ui::messages::text("native.choose_plan_document_markdown"),
    );
}

pub fn pick_executable() -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_EXECUTABLE") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    #[cfg(target_os = "macos")]
    return macos::pick(
        false,
        crate::ui::messages::text("native.choose_an_agent_cli_executable"),
    );
    #[cfg(not(any(target_os = "macos", windows)))]
    return run(file_dialog(true));
    #[cfg(windows)]
    return windows::pick(
        false,
        true,
        crate::ui::messages::text("native.choose_an_agent_cli_executable"),
    );
}

#[cfg(not(any(target_os = "macos", windows)))]
fn run(mut command: Command) -> Result<Option<PathBuf>, String> {
    let output = command.output().map_err(|e| {
        crate::ui::messages::render("native.folder_dialog_unavailable", &[("e", &e.to_string())])
    })?;
    if !output.status.success() {
        return Err(crate::ui::messages::render(
            "native.folder_dialog_failed",
            &[("value1", String::from_utf8_lossy(&output.stderr).trim())],
        ));
    }
    picked(output.stdout)
}

/// DD-90: a UTF-8 BOM or line ending from the picker process must not reach the path.
pub fn picked(stdout: Vec<u8>) -> Result<Option<PathBuf>, String> {
    let text = String::from_utf8(stdout)
        .map_err(|_| crate::ui::messages::text("native.non_utf_8_folder_path"))?;
    let text = text
        .trim_start_matches('\u{feff}')
        .trim_end_matches(['\r', '\n']);
    let normal = crate::git::git_path(std::ffi::OsStr::new(text));
    let text = normal
        .to_str()
        .ok_or(crate::ui::messages::text("native.non_utf_8_folder_path"))?;
    let root = text.len() == 3 && text.as_bytes()[1] == b':' && text.ends_with(['/', '\\']);
    let trimmed = if text.len() > 1 && !root {
        text.trim_end_matches(['/', '\\'])
    } else {
        text
    };
    Ok((!trimmed.is_empty()).then(|| PathBuf::from(trimmed)))
}

#[cfg(not(any(target_os = "macos", windows)))]
fn dialog() -> Command {
    Command::new("false")
}
#[cfg(not(any(target_os = "macos", windows)))]
fn file_dialog(_executable: bool) -> Command {
    Command::new("false")
}
