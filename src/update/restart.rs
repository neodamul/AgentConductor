//! Preserve arguments and the old-process cleanup handoff when restarting an update.
use std::{ffi::OsString, path::Path, process::Command};

pub(super) fn launch(exe: &Path, args: &[OsString], pid: u32) -> Result<(), String> {
    let (command, bundled) = command(exe, args, pid, cfg!(target_os = "macos"));
    execute(command, bundled)
}

fn command(exe: &Path, args: &[OsString], pid: u32, macos: bool) -> (Command, bool) {
    if let Some(bundle) = exe
        .ancestors()
        .find(|p| macos && p.extension().is_some_and(|e| e == "app"))
    {
        // LaunchServices registers the replacement as an app, not a detached CLI process.
        let mut command = crate::subprocess::command("/usr/bin/open");
        command
            .args(["-n", "-a"])
            .arg(bundle)
            .arg("--env")
            .arg(format!("{}={pid}", super::WAIT_PID))
            .arg("--args")
            .args(args);
        return (command, true);
    }
    let mut command = crate::subprocess::command(exe);
    command.args(args).env(super::WAIT_PID, pid.to_string());
    (command, false)
}

fn execute(mut command: Command, bundled: bool) -> Result<(), String> {
    if bundled {
        let status = command
            .status()
            .map_err(|e| format!("cannot restart: {e}"))?;
        if !status.success() {
            return Err(format!("cannot restart the updated app: {status}"));
        }
    } else {
        command
            .spawn()
            .map_err(|e| format!("cannot restart: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/update_restart.rs"]
mod tests;
