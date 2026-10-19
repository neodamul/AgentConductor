//! DD-135: find a newer published release on GitHub and replace this executable with it.
//!
//! Self-replacement: a running executable cannot be overwritten on Windows, but it can be
//! renamed. The installer renames the running file (or macOS bundle) to `*.old`, puts the
//! verified new one in its place, starts it with the old process ID and exits. The new
//! process waits for the old one to end, then removes the `*.old` leftover.
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const REPOSITORY: &str = "neodamul/AgentConductor";
/// Set on the relaunched process: the ID of the process it replaces.
pub const WAIT_PID: &str = "AGENTCONDUCTOR_UPDATE_WAIT_PID";
const COMPANY_SUFFIX: &str = include_str!("../company-suffix.txt");

#[derive(Clone, Default)]
struct Release {
    tag: String,
    page: String,
    notes: String,
    asset: Option<(String, String)>,
    sums: Option<String>,
}

#[derive(Clone, Default)]
struct Status {
    phase: &'static str,
    checked_ms: u64,
    release: Option<Release>,
    error: Option<String>,
}

fn state() -> &'static Mutex<Status> {
    static STATE: OnceLock<Mutex<Status>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(Status {
            phase: "idle",
            ..Default::default()
        })
    })
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

pub fn current() -> String {
    current_with_suffix(env!("CARGO_PKG_VERSION"), COMPANY_SUFFIX)
}

fn current_with_suffix(version: &str, suffix: &str) -> String {
    let suffix = suffix.trim();
    if !suffix.is_empty()
        && suffix.len() <= 64
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        format!("{version}.{suffix}")
    } else {
        version.to_owned()
    }
}

#[cfg(test)]
#[path = "../tests/unit/update.rs"]
mod tests;

/// The release target of this build, as named in the published archives.
pub fn target() -> Option<&'static str> {
    if cfg!(all(windows, target_arch = "x86_64")) {
        Some("x86_64-pc-windows-msvc")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("aarch64-apple-darwin")
    } else {
        None
    }
}

fn version(tag: &str) -> Option<(u64, u64, u64)> {
    let mut parts = tag.trim().trim_start_matches('v').split('.');
    let mut next = || parts.next()?.parse::<u64>().ok();
    Some((next()?, next()?, next()?))
}

/// True when `tag` is a strictly newer x.y.z version than this build.
pub fn is_newer(tag: &str) -> bool {
    matches!((version(tag), version(&current())), (Some(a), Some(b)) if a > b)
}

/// The update state for the page: versions, whether an install is possible, and progress.
pub fn status() -> Value {
    let s = state().lock().map(|s| s.clone()).unwrap_or_default();
    let release = s.release.as_ref();
    let newer = release.is_some_and(|r| is_newer(&r.tag));
    json!({
        "phase": s.phase,
        "current": current(),
        "latest": release.map(|r| r.tag.trim_start_matches('v').to_string()),
        "page": release.map(|r| r.page.clone()),
        "notes": release.map(|r| r.notes.clone()).unwrap_or_default(),
        "newer": newer,
        "installable": newer && release.is_some_and(|r| r.asset.is_some() && r.sums.is_some()),
        "checked_ms": s.checked_ms,
        "error": s.error,
    })
}

fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    let output = crate::subprocess::command("curl")
        .args([
            "-fsSL",
            "--proto",
            "=https",
            "-H",
            "User-Agent: AgentConductor",
        ])
        .args(args)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if error.is_empty() {
            "download failed".into()
        } else {
            error
        });
    }
    Ok(output.stdout)
}

fn latest() -> Result<Release, String> {
    let url = format!("https://api.github.com/repos/{REPOSITORY}/releases/latest");
    let body = curl(&[
        "--max-time",
        "20",
        "-H",
        "Accept: application/vnd.github+json",
        &url,
    ])?;
    let value: Value = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    let tag = value["tag_name"]
        .as_str()
        .ok_or("the latest release has no tag")?;
    let assets = value["assets"].as_array().cloned().unwrap_or_default();
    let url_of = |asset: &Value| asset["browser_download_url"].as_str().map(str::to_owned);
    let asset = target().and_then(|target| {
        let prefix = format!("agentconductor-{target}-");
        assets.iter().find_map(|a| {
            let name = a["name"].as_str()?;
            (name.starts_with(&prefix) && name.ends_with(".zip"))
                .then(|| Some((name.to_owned(), url_of(a)?)))
                .flatten()
        })
    });
    let sums = assets
        .iter()
        .find(|a| a["name"] == "SHA256SUMS")
        .and_then(url_of);
    Ok(Release {
        tag: tag.to_owned(),
        page: value["html_url"].as_str().unwrap_or_default().to_owned(),
        notes: value["body"].as_str().unwrap_or_default().to_owned(),
        asset,
        sums,
    })
}

/// Starts a background check unless one is running; the page reads the result via status().
pub fn check_async() {
    {
        let Ok(mut s) = state().lock() else { return };
        if matches!(s.phase, "checking" | "installing" | "restarting") {
            return;
        }
        s.phase = "checking";
    }
    std::thread::spawn(|| {
        let result = latest();
        if let Ok(mut s) = state().lock() {
            s.phase = "idle";
            s.checked_ms = now_ms();
            match result {
                Ok(release) => {
                    s.release = Some(release);
                    s.error = None;
                }
                Err(error) => s.error = Some(error),
            }
        }
    });
}

/// CLI `update`: installs a newer release over this executable without restarting it.
/// Returns the installed version, or None when this build is already the latest.
mod install;
pub use install::{after_restart, apply_async, install_latest, leftover, replace, verify};
