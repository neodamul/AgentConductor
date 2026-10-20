mod command;
mod forms;

use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Disposition {
    Automatic,
    Human,
    /// Refuse this call only; the worker continues and may choose another action.
    Deny(String),
}

pub struct Policy {
    workspace: PathBuf,
    configured: PathBuf,
    scopes: Vec<String>,
    identity: (u64, u64),
}

fn safe_relative(path: &str) -> Result<(), String> {
    if path.len() > 4096
        || path.contains(['\\', '\0', '\n', '\r'])
        || path.split('/').count() > 64
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("tool path must be an unambiguous bounded workspace-relative path".into());
    }
    if metadata(path) {
        return Err(
            "repository metadata and agent configuration are outside the tool policy".into(),
        );
    }
    Ok(())
}

/// Repository metadata and agent configuration, in any case and either separator.
fn metadata(path: &str) -> bool {
    path.split(['/', '\\']).any(|part| {
        matches!(
            part.to_ascii_lowercase().as_str(),
            ".git" | ".claude" | ".codex" | ".agents" | ".agentconductor" | ".mcp.json"
        )
    })
}

fn sensitive(path: &str) -> bool {
    path.split('/').any(|part| {
        let part = part.to_ascii_lowercase();
        matches!(
            part.as_str(),
            "secrets" | "secret" | ".aws" | ".ssh" | ".gnupg" | ".npmrc" | ".netrc" | ".pypirc"
        ) || part == ".env"
            || part.starts_with(".env.")
            || part.ends_with(".pem")
            || part.ends_with(".key")
            || part.starts_with("credentials")
            || part.starts_with("secrets.")
            || part.starts_with("id_rsa")
            || part.starts_with("id_ed25519")
    })
}

fn string<'a>(input: &'a Value, key: &str, nonempty: bool) -> Result<&'a str, String> {
    input[key]
        .as_str()
        .filter(|s| !nonempty || !s.trim().is_empty())
        .ok_or_else(|| format!("tool input requires a valid {key}"))
}

fn keys(input: &Value, allowed: &[&str]) -> Result<(), String> {
    let fields = input.as_object().ok_or("tool input must be an object")?;
    if fields.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("unrecognized tool input field; execution refused".into());
    }
    Ok(())
}

fn file_metadata(path: &Path, terminal: bool) -> Result<(), String> {
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

fn directory_identity(path: &Path) -> Result<(u64, u64), String> {
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
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
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

mod policy;
