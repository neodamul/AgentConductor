//! Background process I/O without allocating a Windows console window.
use std::{ffi::OsStr, process::Command};
pub fn command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    command
        .env_remove("AC_UI_EVENT_ADDRESS")
        .env_remove("AC_UI_EVENT_TOKEN")
        .env_remove("AC_UI_EVENT_SURFACE");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = &mut command;
    command
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
