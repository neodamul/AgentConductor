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
    let output = command
        .output()
        .map_err(|e| format!("folder dialog unavailable: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "folder dialog failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let text = String::from_utf8(output.stdout).map_err(|_| "non-UTF-8 folder path")?;
    let text = text.trim_end_matches(['\r', '\n']);
    let trimmed = if text.len() > 1 {
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
    let script = windows_owner_script(
        "$d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = 'Choose a folder for AgentConductor'; if ($d.ShowDialog($o) -eq 'OK') { [Console]::Out.Write($d.SelectedPath) }",
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
    let script = windows_owner_script(if executable {
        "$d = New-Object System.Windows.Forms.OpenFileDialog; $d.Title = 'Choose an agent CLI executable'; $d.Filter = 'Executable (*.exe;*.cmd;*.bat)|*.exe;*.cmd;*.bat|All files (*.*)|*.*'; if ($d.ShowDialog($o) -eq 'OK') { [Console]::Out.Write($d.FileName) }"
    } else {
        "$d = New-Object System.Windows.Forms.OpenFileDialog; $d.Title = 'Choose a plan document (Markdown)'; $d.Filter = 'Markdown (*.md;*.markdown)|*.md;*.markdown|Text (*.txt)|*.txt|All files (*.*)|*.*'; if ($d.ShowDialog($o) -eq 'OK') { [Console]::Out.Write($d.FileName) }"
    });
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
    let owner =
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() } as isize;
    format!(
        r#"Add-Type -AssemblyName System.Windows.Forms; Add-Type -ReferencedAssemblies System.Windows.Forms -TypeDefinition 'public sealed class PickerOwner : System.Windows.Forms.IWin32Window {{ public System.IntPtr Handle {{ get; private set; }} public PickerOwner(long handle) {{ Handle = new System.IntPtr(handle); }} }}'; $o = New-Object PickerOwner -ArgumentList ([long]{owner}); {body}"#
    )
}
