use std::{
    fs::File,
    path::{Path, PathBuf},
};

/// A kernel-owned exclusive lease. Keep the inode in place after release so
/// another owner cannot accidentally lock a different file at the same path.
pub struct Lease {
    _file: File,
    path: PathBuf,
}

impl Lease {
    /// Serialize workspace ownership changes for one canonical project folder.
    pub fn project_binding(project: &Path) -> Result<Self, String> {
        let canonical = project.canonicalize().map_err(|e| e.to_string())?;
        Self::scoped("project-binding", canonical.as_os_str().as_encoded_bytes())
    }

    #[cfg(unix)]
    pub fn repository(repo: &Path) -> Result<Self, String> {
        let canonical = repo.canonicalize().map_err(|e| e.to_string())?;
        Self::scoped("repository", canonical.as_os_str().as_encoded_bytes())
    }

    #[cfg(unix)]
    fn scoped(namespace: &str, value: &[u8]) -> Result<Self, String> {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::{DirBuilderExt, MetadataExt};
        // SAFETY: geteuid takes no arguments, touches no memory and cannot fail.
        let uid = unsafe { libc::geteuid() };
        let root = std::env::temp_dir().join(format!("agentconductor-leases-{uid}"));
        match std::fs::DirBuilder::new().mode(0o700).create(&root) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.to_string()),
        }
        let metadata = std::fs::symlink_metadata(&root).map_err(|e| e.to_string())?;
        if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
            return Err(
                "repository lease directory must be private and owned by the current user".into(),
            );
        }
        let key = format!("{:x}", Sha256::digest(value));
        Self::acquire(&root.join(format!("{namespace}-{key}.lock")))
    }

    #[cfg(not(unix))]
    pub fn repository(repo: &Path) -> Result<Self, String> {
        let canonical = repo.canonicalize().map_err(|e| e.to_string())?;
        Self::scoped("repository", canonical.as_os_str().as_encoded_bytes())
    }

    /// The per-user temporary directory is private to the signed-in profile on Windows.
    #[cfg(not(unix))]
    fn scoped(namespace: &str, value: &[u8]) -> Result<Self, String> {
        use sha2::{Digest, Sha256};
        let root = std::env::temp_dir().join("agentconductor-leases");
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let metadata = std::fs::symlink_metadata(&root).map_err(|e| e.to_string())?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("repository lease directory must be a real directory".into());
        }
        let key = format!("{:x}", Sha256::digest(value));
        Self::acquire(&root.join(format!("{namespace}-{key}.lock")))
    }

    /// A stable lock outside a workspace serializes opening against archived deletion.
    pub fn workspace(path: &Path) -> Result<Self, String> {
        let canonical = path.canonicalize().map_err(|e| e.to_string())?;
        Self::scoped(
            "workspace-navigation",
            canonical.as_os_str().as_encoded_bytes(),
        )
    }

    pub fn acquire(path: &Path) -> Result<Self, String> {
        use std::io::Write;
        let mut options = std::fs::OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            // Open a reparse point itself instead of following it.
            options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
        }
        let mut file = options
            .open(path)
            .map_err(|e| format!("open coordinator lease: {e}"))?;
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err("lease must be a regular file".into());
            }
        }
        if !metadata.is_file() {
            return Err("lease must be a regular file".into());
        }
        // The kernel releases this lock when the handle closes, including after a kill.
        match file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err("lease is held: another owner holds the lock".into());
            }
            Err(std::fs::TryLockError::Error(e)) => return Err(format!("lease is held: {e}")),
        }
        file.set_len(0).map_err(|e| e.to_string())?;
        writeln!(file, "{}", std::process::id()).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        Ok(Self {
            _file: file,
            path: path.to_path_buf(),
        })
    }
}

/// A durable run claim prevents a new coordinator from ignoring orphaned agents
/// after the former coordinator's kernel lease has been released.
pub struct WorkspaceLease {
    _lease: Lease,
    record: PathBuf,
    run_dir: PathBuf,
    claimed: bool,
}

impl WorkspaceLease {
    pub fn acquire(repo: &Path, run_dir: &Path) -> Result<Self, String> {
        let lease = Lease::repository(repo)?;
        let record = lease.path.with_extension("owner.json");
        match std::fs::read(&record) {
            Ok(bytes) => {
                let previous: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| format!("invalid repository ownership record: {e}"))?;
                if previous["active"] != false {
                    let path = previous["run_dir"]
                        .as_str()
                        .ok_or("missing prior run identity")?;
                    let report = crate::workflow::load(Path::new(path)).map_err(|e| {
                        format!("prior repository ownership is unknown; investigate {path}: {e}")
                    })?;
                    if report.state == "running"
                        || report.tasks.iter().any(|t| t.status == "running")
                    {
                        return Err(format!(
                            "prior repository coordinator was interrupted; recover {path} before starting another run"
                        ));
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        let run_dir = if run_dir.is_absolute() {
            run_dir.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(run_dir)
        };
        Ok(Self {
            _lease: lease,
            record,
            run_dir,
            claimed: false,
        })
    }

    pub fn claim(&mut self) -> Result<(), String> {
        write_repository_owner(&self.record, &self.run_dir, true)?;
        self.claimed = true;
        Ok(())
    }
}

impl Drop for WorkspaceLease {
    fn drop(&mut self) {
        if self.claimed
            && crate::workflow::load(&self.run_dir).is_ok_and(|r| {
                r.state != "running" && r.tasks.iter().all(|t| t.status != "running")
            })
        {
            let _ = write_repository_owner(&self.record, &self.run_dir, false);
        }
    }
}

fn write_repository_owner(path: &Path, run_dir: &Path, active: bool) -> Result<(), String> {
    write_claim(
        path,
        &serde_json::json!({"active":active,"run_dir":run_dir}),
    )
}

fn write_claim(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    use std::io::Write;
    let temporary = path.with_extension(format!("{}.pending", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    std::fs::rename(temporary, path).map_err(|e| e.to_string())?;
    crate::process::sync_directory(path.parent().ok_or("missing repository lease directory")?)
}

/// Session identity is scoped to the provider/account rather than a profile or
/// checkout. The previous process record must prove quiescence after a crash.
pub struct SessionLease {
    _lease: Lease,
}

impl SessionLease {
    pub fn acquire(key: &str, owner_path: &Path) -> Result<Self, String> {
        let lease = Lease::scoped("session", key.as_bytes())?;
        let record = lease.path.with_extension("owner.json");
        match std::fs::read(&record) {
            Ok(bytes) => {
                let previous: serde_json::Value =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                let owner = Path::new(
                    previous["owner"]
                        .as_str()
                        .ok_or("missing prior session ownership")?,
                );
                let metadata = std::fs::symlink_metadata(owner)
                    .map_err(|e| format!("session ownership unknown: {e}"))?;
                if !metadata.is_file() || metadata.len() > 8192 {
                    return Err("invalid session ownership evidence".into());
                }
                let ownership: crate::process::Ownership =
                    serde_json::from_slice(&std::fs::read(owner).map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?;
                ownership.ensure_quiescent()?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.to_string()),
        }
        let owner = if owner_path.is_absolute() {
            owner_path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(owner_path)
        };
        write_claim(&record, &serde_json::json!({"owner":owner}))?;
        Ok(Self { _lease: lease })
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        // Concurrent process creation may briefly inherit this handle; unlock explicitly
        // so the lease does not outlive this owner.
        let _ = self._file.unlock();
    }
}

#[cfg(windows)]
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
