//! Filesystem-only executable discovery. Never starts a shell or an agent.
use std::path::{Path, PathBuf};
pub fn resolve(name: &str) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    #[cfg(unix)]
    {
        if let Some(home) = std::env::var_os("HOME") {
            for sub in [".local/bin", ".cargo/bin"] {
                dirs.push(PathBuf::from(&home).join(sub));
            }
        }
        dirs.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
        ]);
    }
    // Windows: npm global installs (claude, codex, opencode) and per-user tool folders.
    #[cfg(windows)]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            dirs.push(PathBuf::from(appdata).join("npm"));
        }
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            for sub in [".local\\bin", ".cargo\\bin"] {
                dirs.push(PathBuf::from(&profile).join(sub));
            }
        }
    }
    resolve_in(name, &dirs)
}
/// Keeps symlink names: some provider adapters recognize the executable basename.
pub fn resolve_in(name: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    if name.is_empty() {
        return None;
    }
    let path = Path::new(name);
    let candidates: Vec<PathBuf> = if path.is_absolute() || path.components().count() > 1 {
        vec![path.to_owned()]
    } else {
        dirs.iter().map(|d| d.join(path)).collect()
    };
    for candidate in candidates {
        let mut names = vec![candidate.clone()];
        #[cfg(windows)]
        if candidate.extension().is_none() {
            for ext in ["exe", "cmd", "bat", "com"] {
                names.push(candidate.with_extension(ext));
            }
        }
        for file in names.drain(..) {
            if runnable(&file) {
                return if file.is_absolute() {
                    Some(file)
                } else {
                    std::env::current_dir().ok().map(|cwd| cwd.join(file))
                };
            }
        }
    }
    None
}
fn runnable(path: &Path) -> bool {
    let Ok(metadata) = path.metadata() else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}
