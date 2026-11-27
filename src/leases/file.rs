use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub(super) fn repository_path(repo: &Path, create: bool) -> Result<PathBuf, String> {
    let canonical = repo.canonicalize().map_err(|e| e.to_string())?;
    scoped_path(
        "repository",
        canonical.as_os_str().as_encoded_bytes(),
        create,
    )
}

pub(super) fn scoped_path(namespace: &str, value: &[u8], create: bool) -> Result<PathBuf, String> {
    let root = lease_root(create)?;
    Ok(root.join(format!("{namespace}-{:x}.lock", Sha256::digest(value))))
}

#[cfg(unix)]
fn lease_root(create: bool) -> Result<PathBuf, String> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt};
    // SAFETY: geteuid takes no arguments, touches no memory and cannot fail.
    let uid = unsafe { libc::geteuid() };
    let root = std::env::temp_dir().join(format!("agentconductor-leases-{uid}"));
    if create {
        match fs::DirBuilder::new().mode(0o700).create(&root) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    match fs::symlink_metadata(&root) {
        Ok(metadata)
            if metadata.is_dir() && metadata.uid() == uid && metadata.mode() & 0o077 == 0 =>
        {
            Ok(root)
        }
        Ok(_) => {
            Err("repository lease directory must be private and owned by the current user".into())
        }
        Err(error) if !create && error.kind() == io::ErrorKind::NotFound => Ok(root),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(not(unix))]
fn lease_root(create: bool) -> Result<PathBuf, String> {
    let root = std::env::temp_dir().join("agentconductor-leases");
    if create {
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    }
    match fs::symlink_metadata(&root) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(root),
        Ok(_) => Err("repository lease directory must be a real directory".into()),
        Err(error) if !create && error.kind() == io::ErrorKind::NotFound => Ok(root),
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn open(path: &Path, create: bool) -> io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options
        .read(true)
        .write(true)
        .create(create)
        .truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_OPEN_REPARSE_POINT opens the link rather than following it.
        options.custom_flags(0x0020_0000);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // FILE_ATTRIBUTE_REPARSE_POINT.
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(io::Error::other("lease must be a regular file"));
        }
    }
    if !metadata.is_file() {
        return Err(io::Error::other("lease must be a regular file"));
    }
    Ok(file)
}
