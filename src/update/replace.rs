//! DD-135: swap the running executable or bundle for the new one, and clean up after the restart.
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

/// The installed file or bundle this executable runs from, and its replacement in `package`.
fn targets(exe: &Path, package: &Path) -> Result<(PathBuf, PathBuf), String> {
    if cfg!(target_os = "macos")
        && let Some(bundle) = exe
            .ancestors()
            .find(|p| p.extension().is_some_and(|e| e == "app"))
    {
        let fresh = package.join("AgentConductor.app");
        if !fresh.is_dir() {
            return Err("the archive has no AgentConductor.app".into());
        }
        return Ok((bundle.to_path_buf(), fresh));
    }
    let binary = if cfg!(windows) {
        "agentconductor.exe"
    } else {
        "agentconductor"
    };
    let fresh = package.join(binary);
    if !fresh.is_file() {
        return Err("the archive has no AgentConductor executable".into());
    }
    Ok((exe.to_path_buf(), fresh))
}

pub fn leftover(installed: &Path) -> PathBuf {
    let mut name = installed.file_name().unwrap_or_default().to_os_string();
    name.push(".old");
    installed.with_file_name(name)
}

/// Moves the running file or bundle aside and puts the new one in its place, restoring the
/// original if anything fails.
pub fn replace(exe: &Path, package: &Path) -> Result<(), String> {
    let (installed, fresh) = targets(exe, package)?;
    let old = leftover(&installed);
    remove(&old);
    std::fs::rename(&installed, &old).map_err(|e| format!("cannot move the running app: {e}"))?;
    let placed = if fresh.is_dir() {
        std::fs::rename(&fresh, &installed).or_else(|_| copy_dir(&fresh, &installed))
    } else {
        std::fs::copy(&fresh, &installed)
            .map(|_| ())
            .map_err(|e| e.to_string())
    };
    if let Err(error) = placed {
        remove(&installed);
        let _ = std::fs::rename(&old, &installed);
        return Err(format!("cannot install the update: {error}"));
    }
    #[cfg(unix)]
    if installed.is_file() {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&installed, std::fs::Permissions::from_mode(0o755));
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    let status = crate::subprocess::command("ditto")
        .arg(from)
        .arg(to)
        .status()
        .map_err(|e| e.to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "cannot copy the new app bundle".into())
}

pub(super) fn remove(path: &Path) {
    if path.is_dir() {
        let _ = std::fs::remove_dir_all(path);
    } else {
        let _ = std::fs::remove_file(path);
    }
}

/// Run first in a relaunched process: wait for the replaced one to end, then remove its
/// leftover file or bundle.
pub fn after_restart() {
    let Some(pid) = std::env::var(super::WAIT_PID)
        .ok()
        .and_then(|p| p.parse::<u32>().ok())
    else {
        return;
    };
    // SAFETY: called once at startup before any other thread reads the environment.
    unsafe { std::env::remove_var(super::WAIT_PID) };
    wait_for_exit(pid, Duration::from_secs(60));
    if let Ok(exe) = std::env::current_exe() {
        let installed = exe
            .ancestors()
            .find(|p| cfg!(target_os = "macos") && p.extension().is_some_and(|e| e == "app"))
            .map_or(exe.clone(), Path::to_path_buf);
        remove(&leftover(&installed));
    }
}

#[cfg(windows)]
fn wait_for_exit(pid: u32, limit: Duration) {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject},
    };
    // SAFETY: the handle is checked and closed; waiting does not touch process memory.
    unsafe {
        let handle = OpenProcess(PROCESS_SYNCHRONIZE, 0, pid);
        if !handle.is_null() {
            WaitForSingleObject(handle, limit.as_millis() as u32);
            CloseHandle(handle);
        }
    }
}

#[cfg(not(windows))]
fn wait_for_exit(pid: u32, limit: Duration) {
    let deadline = std::time::Instant::now() + limit;
    // SAFETY: signal 0 only checks whether the process exists.
    while unsafe { libc::kill(pid as i32, 0) } == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
}
