#[cfg(windows)]
use super::windows_job;

/// Contains a child's whole process tree: a POSIX process group on Unix, a Job
/// Object that kills its members when closed on Windows (DD-36).
pub(crate) struct ProcessGroup {
    id: i32,
    #[cfg(windows)]
    job: windows_job::Job,
}

impl ProcessGroup {
    /// Configures `command` so no descendant can start outside containment.
    pub(crate) fn prepare(command: &mut tokio::process::Command) {
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.as_std_mut().process_group(0);
        }
        #[cfg(windows)]
        command.creation_flags(windows_job::CREATE_FLAGS);
    }

    /// Takes ownership of a child spawned from a prepared command.
    pub(crate) fn adopt(child: &tokio::process::Child) -> Result<Self, String> {
        let pid = child.id().ok_or("missing child PID")?;
        let id = i32::try_from(pid).map_err(|_| "child PID out of range")?;
        #[cfg(windows)]
        {
            let handle = child.raw_handle().ok_or("missing child handle")?;
            let job = windows_job::Job::new()?;
            job.assign(handle)?;
            windows_job::resume(handle)?;
            Ok(Self { id, job })
        }
        #[cfg(unix)]
        Ok(Self { id })
    }

    pub(crate) fn id(&self) -> i32 {
        self.id
    }

    /// Requests a graceful stop of the whole tree.
    pub(crate) fn interrupt(&self) {
        #[cfg(unix)]
        self.signal(libc::SIGINT);
        #[cfg(windows)]
        windows_job::ctrl_break(self.id as u32);
    }

    pub(crate) fn terminate(&self) {
        #[cfg(unix)]
        self.signal(libc::SIGTERM);
        #[cfg(windows)]
        windows_job::ctrl_break(self.id as u32);
    }

    pub(crate) fn kill(&self) {
        #[cfg(unix)]
        self.signal(libc::SIGKILL);
        #[cfg(windows)]
        self.job.terminate();
    }

    #[cfg(unix)]
    fn signal(&self, signal: i32) {
        // The child starts a new process group whose ID equals its PID.
        unsafe { libc::kill(-self.id, signal) };
    }
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        self.kill();
    }
}

/// The POSIX signal that ended a process; Windows has none.
pub(crate) fn exit_signal(status: &std::process::ExitStatus) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal()
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}

/// Makes a completed rename durable where the platform supports directory sync.
pub(crate) fn sync_directory(path: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        std::fs::File::open(path)
            .and_then(|f| f.sync_all())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(unix))]
    {
        // NTFS journals renames; a directory cannot be opened as a plain file.
        let _ = path;
        Ok(())
    }
}
