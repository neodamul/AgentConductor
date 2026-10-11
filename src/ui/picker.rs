//! DD-46: native folder selection through tools the operating system ships. The result
//! only fills a form field; saving still validates the path.
use std::path::PathBuf;
use std::process::Command;

pub fn pick_folder() -> Result<Option<PathBuf>, String> {
    if let Some(answer) = std::env::var_os("AGENTCONDUCTOR_PICK_FOLDER") {
        return Ok((!answer.is_empty()).then(|| PathBuf::from(answer)));
    }
    let output = dialog()
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

#[cfg(target_os = "macos")]
fn dialog() -> Command {
    let mut command = Command::new("/usr/bin/osascript");
    // Cancel (-128) is an empty answer, not an error.
    for line in [
        "try",
        "POSIX path of (choose folder with prompt \"Choose a folder for AgentConductor\")",
        "on error number -128",
        "return \"\"",
        "end try",
    ] {
        command.args(["-e", line]);
    }
    command
}

#[cfg(windows)]
fn dialog() -> Command {
    use std::os::windows::process::CommandExt;
    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-STA",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "Add-Type -AssemblyName System.Windows.Forms; $o = New-Object System.Windows.Forms.Form -Property @{TopMost=$true}; $d = New-Object System.Windows.Forms.FolderBrowserDialog; $d.Description = 'Choose a folder for AgentConductor'; if ($d.ShowDialog($o) -eq 'OK') { [Console]::Out.Write($d.SelectedPath) }",
        ])
        .creation_flags(0x0800_0000);
    command
}

#[cfg(not(any(target_os = "macos", windows)))]
fn dialog() -> Command {
    Command::new("false")
}
