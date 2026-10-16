use super::*;

pub fn install_latest() -> Result<Option<String>, String> {
    let release = latest()?;
    if !is_newer(&release.tag) {
        return Ok(None);
    }
    install(&release)?;
    Ok(Some(release.tag.trim_start_matches('v').to_owned()))
}

/// Downloads, verifies and installs the newer release in the background, then restarts.
pub fn apply_async() -> Result<(), String> {
    let release = {
        let mut s = state().lock().map_err(|_| "update state poisoned")?;
        if s.phase != "idle" {
            return Err(crate::ui::messages::text("native.update_already_in_progress").into());
        }
        let release = s
            .release
            .clone()
            .filter(|r| is_newer(&r.tag) && r.asset.is_some() && r.sums.is_some())
            .ok_or(crate::ui::messages::text("native.no_installable_update"))?;
        s.phase = "installing";
        s.error = None;
        release
    };
    std::thread::spawn(move || {
        let result = install(&release).and_then(|exe| restart(&exe));
        if let (Err(error), Ok(mut s)) = (result, state().lock()) {
            s.phase = "idle";
            s.error = Some(error);
        }
    });
    Ok(())
}

fn install(release: &Release) -> Result<PathBuf, String> {
    let (name, url) = release
        .asset
        .clone()
        .ok_or("no archive for this platform")?;
    let sums_url = release.sums.clone().ok_or("release has no SHA256SUMS")?;
    let root = crate::desktop::workspace()?
        .parent()
        .ok_or("invalid data directory")?
        .join("updates")
        .join(&release.tag);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let archive = root.join(&name);
    let path = archive.to_str().ok_or("non-UTF8 update path")?;
    curl(&["--max-time", "900", "-o", path, &url])?;
    let sums =
        String::from_utf8(curl(&["--max-time", "60", &sums_url])?).map_err(|e| e.to_string())?;
    verify(&archive, &name, &sums)?;
    let unpacked = root.join("unpacked");
    std::fs::create_dir_all(&unpacked).map_err(|e| e.to_string())?;
    extract(&archive, &unpacked)?;
    let package = std::fs::read_dir(&unpacked)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .ok_or("the archive has no package folder")?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    replace(&exe, &package)?;
    Ok(exe)
}

/// The archive's SHA-256 must match its line in the release's SHA256SUMS.
pub fn verify(archive: &Path, name: &str, sums: &str) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let expected = sums
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file = parts.next()?.trim_start_matches('*');
            (file.rsplit('/').next() == Some(name)).then(|| hash.to_ascii_lowercase())
        })
        .ok_or("the archive is not listed in SHA256SUMS")?;
    let bytes = std::fs::read(archive).map_err(|e| e.to_string())?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err("the downloaded archive does not match its published checksum".into());
    }
    Ok(())
}

fn extract(archive: &Path, into: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        let mut c = crate::subprocess::command("ditto");
        c.args(["-x", "-k"]).arg(archive).arg(into);
        c
    } else {
        // Windows 10 and later ship bsdtar, which reads ZIP archives.
        let mut c = crate::subprocess::command("tar");
        c.arg("-xf").arg(archive).arg("-C").arg(into);
        c
    };
    let output = command.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "cannot unpack the update: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

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

fn remove(path: &Path) {
    if path.is_dir() {
        let _ = std::fs::remove_dir_all(path);
    } else {
        let _ = std::fs::remove_file(path);
    }
}

/// Starts the installed executable with the same arguments and ends this process.
fn restart(exe: &Path) -> Result<(), String> {
    if let Ok(mut s) = state().lock() {
        s.phase = "restarting";
    }
    std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .env(WAIT_PID, std::process::id().to_string())
        .spawn()
        .map_err(|e| format!("cannot restart: {e}"))?;
    std::process::exit(0);
}

/// Run first in a relaunched process: wait for the replaced one to end, then remove its
/// leftover file or bundle.
pub fn after_restart() {
    let Some(pid) = std::env::var(WAIT_PID)
        .ok()
        .and_then(|p| p.parse::<u32>().ok())
    else {
        return;
    };
    // SAFETY: called once at startup before any other thread reads the environment.
    unsafe { std::env::remove_var(WAIT_PID) };
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
