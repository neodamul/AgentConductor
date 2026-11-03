use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

pub struct ValidatedPackage {
    pub binary: PathBuf,
    pub manifest: Value,
}

pub fn validate_package(package: &Path) -> Result<ValidatedPackage, String> {
    let package = package
        .canonicalize()
        .map_err(|error| format!("resolve package directory: {error}"))?;
    if !package.is_dir() {
        return Err("package must be a directory".into());
    }
    let manifest: Value = serde_json::from_slice(
        &fs::read(package.join("manifest.json"))
            .map_err(|error| format!("read manifest: {error}"))?,
    )
    .map_err(|error| format!("parse package manifest: {error}"))?;
    let name = manifest["binary"]
        .as_str()
        .ok_or_else(|| "manifest binary name is missing".to_owned())?;
    if !matches!(name, "agentconductor" | "agentconductor.exe") {
        return Err("invalid packaged binary path".into());
    }
    let binary = package.join(name);
    let metadata =
        fs::symlink_metadata(&binary).map_err(|error| format!("read packaged binary: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("packaged binary must be a regular file".into());
    }
    let digest = sha256_file(&binary)?;
    if manifest["binary_sha256"].as_str() != Some(&digest) {
        return Err("packaged binary checksum mismatch".into());
    }
    let commit = manifest["source_commit"]
        .as_str()
        .ok_or_else(|| "exact packaged source commit required".to_owned())?;
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("exact packaged source commit required".into());
    }
    let sums = fs::read_to_string(package.join("SHA256SUMS"))
        .map_err(|error| format!("read package checksum file: {error}"))?;
    if sums != format!("{digest}  {name}\n") {
        return Err("package checksum file mismatch".into());
    }
    Ok(ValidatedPackage { binary, manifest })
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| format!("open packaged binary: {error}"))?;
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    loop {
        let length = file
            .read(&mut buffer)
            .map_err(|error| format!("read packaged binary: {error}"))?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
