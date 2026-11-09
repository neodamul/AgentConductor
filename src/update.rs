//! DD-135: find a newer published release on GitHub and replace this executable with it.
//!
//! Self-replacement: a running executable cannot be overwritten on Windows, but it can be
//! renamed. The installer renames the running file (or macOS bundle) to `*.old`, puts the
//! verified new one in its place, starts it with the old process ID and exits. The new
//! process waits for the old one to end, then removes the `*.old` leftover.
mod install;
mod replace;

use install::install;
pub use install::verify;
use replace::remove;
pub use replace::{after_restart, leftover, replace};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

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

/// x.y.z plus an optional `-Beta1` style prerelease. The key orders a prerelease below its
/// release, and `Beta2` above `Beta1` and `Beta10` above `Beta9`. A local `.suffix` is ignored.
type Version = (u64, u64, u64, bool, String, u64);

fn version(tag: &str) -> Option<Version> {
    let mut parts = tag.trim().trim_start_matches('v').split('.');
    let mut next = || parts.next()?.parse::<u64>().ok();
    let (major, minor) = (next()?, next()?);
    let third = parts.next()?;
    let digits = third.len() - third.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let patch = third[..digits].parse().ok()?;
    let rest = &third[digits..];
    if rest.is_empty() {
        return Some((major, minor, patch, true, String::new(), 0));
    }
    let pre = rest
        .strip_prefix('-')
        .filter(|pre| !pre.is_empty())?
        .to_ascii_lowercase();
    let split = pre.find(|c: char| c.is_ascii_digit()).unwrap_or(pre.len());
    let number = pre[split..].parse().unwrap_or(0);
    Some((major, minor, patch, false, pre[..split].to_owned(), number))
}

/// True when `tag` is a strictly newer version than this build (a release outranks its betas).
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
    let url = crate::update_source::api_url();
    let body = curl(&[
        "--max-time",
        "20",
        "-H",
        "Accept: application/vnd.github+json",
        &url,
    ])?;
    let value: Value = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    // The beta channel gets a list: the highest published version wins, drafts never count.
    let value = match value.as_array() {
        Some(list) => list
            .iter()
            .filter(|r| r["draft"] != true)
            .filter_map(|r| Some((version(r["tag_name"].as_str()?)?, r)))
            .max_by(|a, b| a.0.cmp(&b.0))
            .map(|(_, r)| r.clone())
            .ok_or("no release was found")?,
        None => value,
    };
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
pub fn install_latest() -> Result<Option<String>, String> {
    let release = latest()?;
    if !is_newer(&release.tag) {
        return Ok(None);
    }
    install(&release)?;
    Ok(Some(release.tag.trim_start_matches('v').to_owned()))
}

/// Downloads, verifies and installs the newer release in the background, then restarts.
pub fn apply_async() -> Result<(), String> {
    let release = {
        let mut s = state().lock().map_err(|_| "update state poisoned")?;
        if s.phase != "idle" {
            return Err(crate::ui::messages::text("native.update_already_in_progress").into());
        }
        let release = s
            .release
            .clone()
            .filter(|r| is_newer(&r.tag) && r.asset.is_some() && r.sums.is_some())
            .ok_or(crate::ui::messages::text("native.no_installable_update"))?;
        s.phase = "installing";
        s.error = None;
        release
    };
    std::thread::spawn(move || {
        let result = install(&release).and_then(|exe| restart(&exe));
        if let (Err(error), Ok(mut s)) = (result, state().lock()) {
            s.phase = "idle";
            s.error = Some(error);
        }
    });
    Ok(())
}

/// Removes every downloaded release folder under `updates`, keeping the folder itself.
pub fn clear_downloads(updates: &Path) {
    let Ok(entries) = std::fs::read_dir(updates) else {
        return;
    };
    for entry in entries.flatten() {
        remove(&entry.path());
    }
}

/// Starts the installed executable with the same arguments and ends this process.
fn restart(exe: &Path) -> Result<(), String> {
    if let Ok(mut s) = state().lock() {
        s.phase = "restarting";
    }
    std::process::Command::new(exe)
        .args(std::env::args_os().skip(1))
        .env(WAIT_PID, std::process::id().to_string())
        .spawn()
        .map_err(|e| format!("cannot restart: {e}"))?;
    std::process::exit(0);
}
