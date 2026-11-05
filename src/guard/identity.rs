//! DD-37: kernel file identity and alias checks for tool paths.
use std::{fs, path::Path};

pub(super) fn file_metadata(path: &Path, terminal: bool) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if metadata.file_type().is_symlink()
        || (terminal && !metadata.is_file())
        || (!terminal && !metadata.is_dir())
    {
        return Err("tool path contains an alias or unsupported filesystem object".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if terminal && metadata.nlink() != 1 {
            return Err("hard-linked tool files are outside the policy".into());
        }
    }
    #[cfg(windows)]
    {
        // Junctions and mount points are reparse points that are not reported as symlinks.
        let identity = windows_identity(path)?;
        if identity.reparse {
            return Err("tool path contains an alias or unsupported filesystem object".into());
        }
        if terminal && identity.links != 1 {
            return Err("hard-linked tool files are outside the policy".into());
        }
    }
    Ok(())
}

pub(super) fn directory_identity(path: &Path) -> Result<(u64, u64), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(windows)]
    {
        let identity = windows_identity(path)?;
        Ok((u64::from(identity.volume), identity.index))
    }
}

/// Kernel file identity read from a handle that does not follow reparse points (DD-37).
#[cfg(windows)]
pub(crate) struct WindowsIdentity {
    pub volume: u32,
    pub index: u64,
    pub links: u32,
    pub reparse: bool,
    pub attributes: u32,
    pub size: u64,
    pub created: u64,
    pub written: u64,
}

#[cfg(windows)]
pub(crate) fn windows_identity(path: &Path) -> Result<WindowsIdentity, String> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, GetFileInformationByHandle,
    };
    let file = fs::OpenOptions::new()
        .access_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .map_err(|e| e.to_string())?;
    // SAFETY: BY_HANDLE_FILE_INFORMATION is a plain C struct of integers, valid when all-zero.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    // SAFETY: `file` is an open handle owned for the call and `info` is a valid, writable struct.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(format!(
            "file identity cannot be read: {}",
            std::io::Error::last_os_error()
        ));
    }
    let time = |t: windows_sys::Win32::Foundation::FILETIME| {
        (u64::from(t.dwHighDateTime) << 32) | u64::from(t.dwLowDateTime)
    };
    Ok(WindowsIdentity {
        volume: info.dwVolumeSerialNumber,
        index: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
        links: info.nNumberOfLinks,
        reparse: info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0,
        attributes: info.dwFileAttributes,
        size: (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow),
        created: time(info.ftCreationTime),
        written: time(info.ftLastWriteTime),
    })
}
