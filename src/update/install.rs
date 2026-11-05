//! DD-135: download, verify, unpack and put a published release in place.
use super::{Release, clear_downloads, curl, replace};
use std::path::{Path, PathBuf};

pub(super) fn install(release: &Release) -> Result<PathBuf, String> {
    let (name, url) = release
        .asset
        .clone()
        .ok_or("no archive for this platform")?;
    let sums_url = release.sums.clone().ok_or("release has no SHA256SUMS")?;
    let root = crate::desktop::workspace()?
        .parent()
        .ok_or("invalid data directory")?
        .join("updates")
        .join(&release.tag);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let archive = root.join(&name);
    let path = archive.to_str().ok_or("non-UTF8 update path")?;
    curl(&["--max-time", "900", "-o", path, &url])?;
    let sums =
        String::from_utf8(curl(&["--max-time", "60", &sums_url])?).map_err(|e| e.to_string())?;
    verify(&archive, &name, &sums)?;
    let unpacked = root.join("unpacked");
    std::fs::create_dir_all(&unpacked).map_err(|e| e.to_string())?;
    extract(&archive, &unpacked)?;
    let package = std::fs::read_dir(&unpacked)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .ok_or("the archive has no package folder")?;
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    replace(&exe, &package)?;
    // The archive and unpacked package are no longer needed; earlier tags are stale too (#80).
    if let Some(updates) = root.parent() {
        clear_downloads(updates);
    }
    Ok(exe)
}

/// The archive's SHA-256 must match its line in the release's SHA256SUMS.
pub fn verify(archive: &Path, name: &str, sums: &str) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let expected = sums
        .lines()
        .find_map(|line| {
            let mut parts = line.split_whitespace();
            let hash = parts.next()?;
            let file = parts.next()?.trim_start_matches('*');
            (file.rsplit('/').next() == Some(name)).then(|| hash.to_ascii_lowercase())
        })
        .ok_or("the archive is not listed in SHA256SUMS")?;
    let bytes = std::fs::read(archive).map_err(|e| e.to_string())?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err("the downloaded archive does not match its published checksum".into());
    }
    Ok(())
}

fn extract(archive: &Path, into: &Path) -> Result<(), String> {
    let mut command = if cfg!(target_os = "macos") {
        let mut c = crate::subprocess::command("ditto");
        c.args(["-x", "-k"]).arg(archive).arg(into);
        c
    } else {
        // Windows 10 and later ship bsdtar, which reads ZIP archives.
        let mut c = crate::subprocess::command("tar");
        c.arg("-xf").arg(archive).arg("-C").arg(into);
        c
    };
    let output = command.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "cannot unpack the update: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}
