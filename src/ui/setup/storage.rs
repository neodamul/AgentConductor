//! Bounded, atomic persistence of startup state.
use crate::ui::messages;
use serde::Serialize;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub(super) fn bytes(path: &Path) -> Result<Vec<u8>, String> {
    let file = crate::ui::open_artifact(path)?;
    let mut data = Vec::new();
    file.take(65537)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 65536 {
        return Err(messages::text("native.startup_document_exceeds_64_kib").into());
    }
    Ok(data)
}
pub(super) fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    serde_json::from_slice(&bytes(path)?).map_err(|e| e.to_string())
}
pub(super) fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if path.exists() || fs::symlink_metadata(path).is_ok() {
        crate::ui::open_artifact(path)?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    if bytes.len() > 64 * 1024 {
        return Err(messages::text("native.startup_document_exceeds_64_kib").into());
    }
    let dir = path
        .parent()
        .ok_or(messages::text("native.missing_settings_directory"))?;
    let pending = dir.join(format!("{}.pending", crate::native_guard::session_id()?));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    fs::rename(&pending, path).map_err(|e| e.to_string())?;
    crate::process::sync_directory(dir)
}
