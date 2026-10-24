use crate::plan::AgentProfile;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_BYTES: usize = 65_536;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CustomProfileRegistry {
    pub version: u8,
    pub profiles: Vec<AgentProfile>,
}

impl CustomProfileRegistry {
    pub fn global_path() -> Result<PathBuf, String> {
        Ok(crate::desktop::workspace()?
            .parent()
            .ok_or("invalid AgentConductor data directory")?
            .join("cli-profiles.json"))
    }

    pub fn load_global() -> Result<Vec<AgentProfile>, String> {
        let path = Self::global_path()?;
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(_) => Err("cannot inspect custom CLI registry".into()),
            Ok(_) => Self::load(&path).map(|registry| registry.profiles),
        }
    }

    pub fn append_templates(
        mut builtins: Vec<AgentProfile>,
        custom: Vec<AgentProfile>,
    ) -> Result<Vec<AgentProfile>, String> {
        if custom
            .iter()
            .any(|profile| builtins.iter().any(|builtin| builtin.id == profile.id))
        {
            return Err("custom CLI registry ID conflicts with a built-in profile".into());
        }
        builtins.extend(custom);
        Ok(builtins)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err("custom CLI registry symlinks are not allowed".into());
        }
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.custom_flags(
                windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT,
            );
        }
        let file = options
            .open(path)
            .map_err(|_| "cannot read custom CLI registry")?;
        let metadata = file
            .metadata()
            .map_err(|_| "cannot inspect custom CLI registry")?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_BYTES as u64
        {
            return Err("custom CLI registry must be a regular file under 64 KiB".into());
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take((MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|_| "cannot read custom CLI registry")?;
        if bytes.len() > MAX_BYTES || bytes.len() as u64 != metadata.len() {
            return Err("custom CLI registry changed or exceeds 64 KiB".into());
        }
        Self::parse(&bytes)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err("custom CLI registry exceeds 64 KiB".into());
        }
        let registry: Self =
            serde_json::from_slice(bytes).map_err(|_| "invalid custom CLI registry JSON")?;
        if registry.version != 1 || registry.profiles.len() > 16 {
            return Err("unsupported custom CLI registry version or profile count".into());
        }
        if registry
            .profiles
            .iter()
            .any(|profile| profile.parser != crate::protocol::Provider::Custom)
        {
            return Err("registry entries must use the custom CLI protocol".into());
        }
        if registry.profiles.iter().any(|profile| {
            matches!(profile.id.as_str(), "claude" | "codex" | "opencode")
                || !profile.environment.is_empty()
        }) {
            return Err(
                "custom registry IDs must be unique and cannot store environment secrets".into(),
            );
        }
        crate::ui::setup::validate_profiles(&registry.profiles)?;
        Ok(registry)
    }
}
