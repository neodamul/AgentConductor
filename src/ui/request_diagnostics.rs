//! Bounded, content-free diagnostics for slow native UI requests.
use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub const MAX_BYTES: usize = 256 * 1024;

pub fn dispatch<T>(bytes: &[u8], execute: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    dispatch_with(bytes, execute, |kind, size, at, elapsed, succeeded| {
        record_default(kind, size, at, elapsed, succeeded);
        Ok::<(), ()>(())
    })
}

pub fn dispatch_with<T, E>(
    bytes: &[u8],
    execute: impl FnOnce() -> Result<T, String>,
    record: impl FnOnce(&str, usize, u64, Duration, bool) -> Result<(), E>,
) -> Result<T, String> {
    let started = std::time::Instant::now();
    let started_at = unix_millis();
    let kind = serde_json::from_slice::<serde_json::Value>(bytes)
        .ok()
        .and_then(|request| request["kind"].as_str().map(str::to_owned));
    let result = execute();
    if let Some(kind) = kind {
        let _ = record(
            &kind,
            bytes.len(),
            started_at,
            started.elapsed(),
            result.is_ok(),
        );
    }
    result
}

pub fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub fn record_default(
    kind: &str,
    bytes: usize,
    at_unix_ms: u64,
    elapsed: Duration,
    succeeded: bool,
) {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if let Some(home) = home {
        let path = Path::new(&home)
            .join(".agentconductor")
            .join("request-diagnostics.log");
        let _ = record_at(&path, kind, bytes, at_unix_ms, elapsed, succeeded);
    }
}

/// Append one request's two JSONL records when it is a chat or took at least one second.
pub fn record_at(
    path: &Path,
    kind: &str,
    bytes: usize,
    at_unix_ms: u64,
    elapsed: Duration,
    succeeded: bool,
) -> bool {
    if kind != "chat" && elapsed < Duration::from_secs(1) {
        return false;
    }
    let mut records = Vec::new();
    for record in [
        json!({"event":"start","kind":kind,"at_unix_ms":at_unix_ms,"bytes":bytes}),
        json!({"event":"done","kind":kind,"duration_ms":elapsed.as_millis(),"outcome":if succeeded {"ok"} else {"error"}}),
    ] {
        let Ok(mut line) = serde_json::to_vec(&record) else {
            return false;
        };
        line.push(b'\n');
        records.extend(line);
    }
    append_bounded(path, &records).is_ok()
}

fn append_bounded(path: &Path, records: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or(std::io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(parent)?;
    let current = fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if current.saturating_add(records.len() as u64) > MAX_BYTES as u64 {
        let backup = path.with_extension("log.1");
        let _ = fs::remove_file(&backup);
        if path.exists() {
            fs::rename(path, backup)?;
        }
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?
        .write_all(records)
}
