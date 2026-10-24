//! An owned, independent coordinator restores durable timers after UI/process restart.
use super::*;
use std::process::Stdio;
fn clock() -> Result<i64, String> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64)
}
fn logs(dir: &Path) -> PathBuf {
    dir.join("availability-coordinator/process")
}
fn owner() -> process::Ownership {
    process::Ownership {
        coordinator_pid: std::process::id(),
        pid: Some(std::process::id()),
        process_group: Some(std::process::id() as i32),
        phase: "running".into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    }
}
pub fn ensure_availability_watcher(dir: &Path, exe: &Path) -> Result<bool, String> {
    // Review/completed runs have no timer work; snapshots must not claim the approving child's lock.
    let report = load(dir)?;
    if matches!(
        report.state.as_str(),
        "waiting_approval" | "approved" | "completed" | "rejected"
    ) {
        return Ok(false);
    }
    let entries = availability_timer(&report, clock()?);
    if entries.is_empty() || dir.join("stop.request").exists() {
        return Ok(false);
    }
    let _spawn = match crate::leases::Lease::acquire(&dir.join("availability-spawn.lock")) {
        Ok(lease) => lease,
        Err(_) => return Ok(true),
    };
    let log = logs(dir);
    let prior = log.join("owner.json");
    if prior.is_file() {
        let o: process::Ownership =
            serde_json::from_slice(&fs::read(&prior).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if o.ensure_quiescent().is_err() {
            return Ok(true);
        }
    }
    // A child may hold the lease before publishing its ownership record.
    match crate::leases::Lease::acquire(&dir.join("availability-watch.lock")) {
        Ok(lease) => drop(lease),
        Err(_) => return Ok(true),
    }
    fs::create_dir_all(&log).map_err(|e| e.to_string())?;
    let mut command = crate::subprocess::command(exe);
    command
        .arg("availability-watch")
        .arg(dir)
        .stdin(Stdio::null())
        .stdout(fs::File::create(log.join("stdout.log")).map_err(|e| e.to_string())?)
        .stderr(fs::File::create(log.join("stderr.log")).map_err(|e| e.to_string())?);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(
            windows_sys::Win32::System::Threading::CREATE_NO_WINDOW
                | windows_sys::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP,
        );
    }
    let mut child = {
        // std::process on Windows also inherits unrelated inheritable caller pipes.
        // Child stdio is explicitly Null/files; restore caller flags after spawning.
        #[cfg(windows)]
        let _spawn_serialization = {
            static SPAWN: std::sync::Mutex<()> = std::sync::Mutex::new(());
            SPAWN.lock().map_err(|_| "watcher spawn lock poisoned")?
        };
        #[cfg(windows)]
        let _caller_pipes = prevent_caller_pipe_inheritance()?;
        command.spawn().map_err(|e| e.to_string())?
    };
    let mut ownership = owner();
    ownership.pid = Some(child.id());
    ownership.process_group = Some(child.id() as i32);
    process::record_owner(&log, &ownership)?;
    let reaper_log = log.clone();
    std::thread::spawn(move || {
        if let Ok(status) = child.wait() {
            ownership.phase = "terminal".into();
            ownership.exit_code = status.code();
            ownership.termination_signal = crate::process::exit_signal(&status);
            let _ = process::record_owner(&reaper_log, &ownership);
        }
    });
    Ok(true)
}
pub fn watch_availability(dir: &Path) -> Result<(), String> {
    let _lease = match crate::leases::Lease::acquire(&dir.join("availability-watch.lock")) {
        Ok(lease) => lease,
        Err(_) => return Ok(()),
    };
    let claim = loop {
        match crate::leases::Lease::acquire(&dir.join("availability-spawn.lock")) {
            Ok(lease) => break lease,
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    };
    let log = logs(dir);
    fs::create_dir_all(&log).map_err(|e| e.to_string())?;
    let mut owned = owner();
    process::record_owner(&log, &owned)?;
    drop(claim);
    let result = (|| {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        loop {
            let current = clock()?;
            // Idle polling reads durable state; only due execution claims the workflow lease.
            let entries = super::availability_timer(&load(dir)?, current);
            if entries.is_empty() || dir.join("stop.request").exists() {
                break;
            }
            if entries.iter().any(|t| t.at <= current) {
                // Resume re-reads the current assignment under the exclusive workflow lock.
                // A reassign/Stop before that claim therefore cancels the old task timer.
                let report =
                    runtime.block_on(super::resume_availability_due(dir, current, 1800))?;
                if report.state != "waiting_availability" {
                    let _ = runtime.block_on(super::repair(dir, 1800, report))?;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        Ok::<_, String>(())
    })();
    owned.phase = "terminal".into();
    owned.exit_code = Some(if result.is_ok() { 0 } else { 1 });
    process::record_owner(&log, &owned)?;
    Store::open(dir.join("state.sqlite"))?.record("availability_coordinator",&serde_json::json!({"pid":std::process::id(),"status":if result.is_ok(){"finished"}else{"failed"},"error":result.as_ref().err()}),"Availability coordinator exited; reservations are restored from current run state on reopening")?;
    result
}

#[cfg(windows)]
struct PipeInheritance(Vec<(windows_sys::Win32::Foundation::HANDLE, u32)>);
#[cfg(windows)]
impl Drop for PipeInheritance {
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};
        for &(handle, flags) in &self.0 {
            // SAFETY: `handle` is a standard handle recorded by `prevent_caller_pipe_inheritance`; standard handles stay open for the process lifetime.
            unsafe {
                SetHandleInformation(handle, HANDLE_FLAG_INHERIT, flags & HANDLE_FLAG_INHERIT);
            }
        }
    }
}
#[cfg(windows)]
fn prevent_caller_pipe_inheritance() -> Result<PipeInheritance, String> {
    use windows_sys::Win32::{
        Foundation::{GetHandleInformation, HANDLE_FLAG_INHERIT, SetHandleInformation},
        Storage::FileSystem::{FILE_TYPE_PIPE, GetFileType},
        System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE},
    };
    let mut restore = PipeInheritance(vec![]);
    for stream in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        // SAFETY: GetStdHandle takes a constant stream id and returns a handle or an invalid value, which GetFileType rejects below.
        let handle = unsafe { GetStdHandle(stream) };
        // SAFETY: GetFileType only reads the handle; an invalid handle yields FILE_TYPE_UNKNOWN, which skips it.
        if unsafe { GetFileType(handle) } != FILE_TYPE_PIPE {
            continue;
        }
        let mut flags = 0;
        // SAFETY: `handle` is a pipe handle (checked above) and `flags` is a valid, writable u32.
        if unsafe { GetHandleInformation(handle, &mut flags) } == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        if flags & HANDLE_FLAG_INHERIT != 0 {
            // SAFETY: `handle` is a standard pipe handle that stays open; only its inherit flag changes.
            if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0) } == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            restore.0.push((handle, flags));
        }
    }
    Ok(restore)
}
