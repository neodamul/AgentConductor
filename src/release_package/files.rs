use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub(super) fn referenced_assets(root: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let readme = fs::read_to_string(root.join("README.md"))
        .map_err(|error| format!("read README: {error}"))?;
    let docs = root
        .join("docs")
        .canonicalize()
        .map_err(|error| format!("resolve docs: {error}"))?;
    let mut assets = BTreeMap::new();
    let mut rest = readme.as_str();
    while let Some(end) = rest.find("](") {
        rest = &rest[end + 2..];
        let Some(close) = rest.find(')') else { break };
        let raw = rest[..close].trim();
        let raw = raw
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
            .unwrap_or(raw);
        let relative = raw.split_ascii_whitespace().next().unwrap_or("");
        if relative.starts_with("docs/") {
            let source = root
                .join(relative)
                .canonicalize()
                .map_err(|_| format!("missing or invalid README asset: {relative}"))?;
            if !source.starts_with(&docs) || !source.is_file() {
                return Err(format!("missing or invalid README asset: {relative}"));
            }
            assets.insert(relative.to_owned(), source);
        }
        rest = &rest[close + 1..];
    }
    Ok(assets)
}

pub(super) fn project_version(manifest: &Path) -> Result<String, String> {
    let text = fs::read_to_string(manifest).map_err(|error| error.to_string())?;
    let mut in_package = false;
    for line in text.lines() {
        if line.starts_with('[') {
            in_package = line.trim() == "[package]";
        } else if in_package && let Some(value) = line.trim().strip_prefix("version = ") {
            return Ok(value.trim().trim_matches('"').to_owned());
        }
    }
    Err("package version missing from Cargo.toml".into())
}

pub(super) fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("cargo metadata omitted {key}"))
}

pub(super) fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::copy(source, destination)
        .map(|_| ())
        .map_err(|error| format!("copy {}: {error}", source.display()))
}

pub(super) fn write_bytes(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let path = root.join(name);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

pub(super) fn write_json(root: &Path, name: &str, value: &Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    write_bytes(root, name, &bytes)
}

pub(super) fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
