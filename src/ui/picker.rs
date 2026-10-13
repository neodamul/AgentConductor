//! DD-46: native folder selection through tools the operating system ships. The result
//! only fills a form field; saving still validates the path.
#[cfg(target_os = "macos")]
#[path = "picker/macos.rs"]
mod macos;
use std::path::PathBuf;
#[cfg(not(target_os = "macos"))]
use std::process::Command;

pub fn pick_folder() -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_FOLDER") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    #[cfg(target_os = "macos")]
    return macos::pick(true, false);
    #[cfg(not(target_os = "macos"))]
    run(dialog())
}

/// DD-60: choose a Markdown plan document; the result is only a path to read.
pub fn pick_file() -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_FILE") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    #[cfg(target_os = "macos")]
    return macos::pick(false, false);
    #[cfg(not(target_os = "macos"))]
    run(file_dialog(false))
}

pub fn pick_executable() -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_EXECUTABLE") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    #[cfg(target_os = "macos")]
    return macos::pick(false, true);
    #[cfg(not(target_os = "macos"))]
    run(file_dialog(true))
}

#[cfg(not(target_os = "macos"))]
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

#[cfg(windows)]
fn dialog() -> Command {
    use std::os::windows::process::CommandExt;
    let mut command = Command::new("powershell.exe");
    command.env(
        "AGENTCONDUCTOR_PICKER_TITLE",
        crate::ui::messages::text("native.choose_folder_for_agentconductor"),
    );
    let script = windows_owner_script(
        "$d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = $env:AGENTCONDUCTOR_PICKER_TITLE; if ($d.ShowDialog($o) -eq 'OK') { [Console]::Out.Write($d.SelectedPath) }",
    );
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-STA",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .creation_flags(0x0800_0000);
    command
}

#[cfg(not(any(target_os = "macos", windows)))]
fn dialog() -> Command {
    Command::new("false")
}

#[cfg(windows)]
fn file_dialog(executable: bool) -> Command {
    use std::os::windows::process::CommandExt;
    let mut command = Command::new("powershell.exe");
    let (title, filter) = if executable {
        (
            crate::ui::messages::text("native.choose_an_agent_cli_executable"),
            crate::ui::messages::text("native.executable_filter"),
        )
    } else {
        (
            crate::ui::messages::text("native.choose_plan_document_markdown"),
            crate::ui::messages::text("native.document_filter"),
        )
    };
    command
        .env("AGENTCONDUCTOR_PICKER_TITLE", title)
        .env("AGENTCONDUCTOR_PICKER_FILTER", filter);
    let script = windows_owner_script(
        "$d = New-Object System.Windows.Forms.OpenFileDialog; $d.Title = $env:AGENTCONDUCTOR_PICKER_TITLE; $d.Filter = $env:AGENTCONDUCTOR_PICKER_FILTER; if ($d.ShowDialog($o) -eq 'OK') { [Console]::Out.Write($d.FileName) }",
    );
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-STA",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .creation_flags(0x0800_0000);
    command
}

#[cfg(not(any(target_os = "macos", windows)))]
fn file_dialog(_executable: bool) -> Command {
    Command::new("false")
}

#[cfg(windows)]
pub(crate) fn windows_owner_script(body: &str) -> String {
    // Capture the app's foreground HWND before starting the short-lived picker process.
    // UTF-8 without BOM: the default OEM code page garbles non-ASCII paths (DD-90).
    let owner =
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() } as isize;
    format!(
        r#"[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false; Add-Type -AssemblyName System.Windows.Forms; Add-Type -ReferencedAssemblies System.Windows.Forms -TypeDefinition 'public sealed class PickerOwner : System.Windows.Forms.IWin32Window {{ public System.IntPtr Handle {{ get; private set; }} public PickerOwner(long handle) {{ Handle = new System.IntPtr(handle); }} }}'; $o = New-Object PickerOwner -ArgumentList ([long]{owner}); {body}"#
    )
}
