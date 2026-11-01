//! Background process I/O without allocating a Windows console window.
use std::{ffi::OsStr, process::Command};
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = &mut command;
    command
}
/// Start chat work independently so the result can be reattached after the UI closes.
pub fn background_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::{
            CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW,
        };
        command.creation_flags(
            CREATE_BREAKAWAY_FROM_JOB | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW,
        );
    }
    #[cfg(not(windows))]
    let _ = &mut command;
    command
}

/// Fallback when the host process job refuses Windows breakaway.
pub fn background_fallback_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW};
        command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = &mut command;
    command
}

pub fn background_fallback_allowed(is_in_host_job: Option<bool>) -> bool {
    is_in_host_job == Some(false)
}

#[cfg(windows)]
pub fn current_process_is_in_host_job() -> std::io::Result<bool> {
    use windows_sys::Win32::System::{JobObjects::IsProcessInJob, Threading::GetCurrentProcess};
    let mut is_in_job = 0;
    // SAFETY: GetCurrentProcess returns a valid pseudo-handle; the output pointer is writable.
    let succeeded =
        unsafe { IsProcessInJob(GetCurrentProcess(), std::ptr::null_mut(), &mut is_in_job) };
    if succeeded == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(is_in_job != 0)
    }
}

#[cfg(not(windows))]
pub fn current_process_is_in_host_job() -> std::io::Result<bool> {
    Ok(false)
}

/// Reuse a calling terminal for CLI commands; never allocate a new console.
pub fn attach_parent_console() {
    #[cfg(windows)]
    // SAFETY: AttachConsole takes a constant process id and no pointers; failure is harmless and the result is ignored.
    unsafe {
        windows_sys::Win32::System::Console::AttachConsole(
            windows_sys::Win32::System::Console::ATTACH_PARENT_PROCESS,
        );
    }
}
