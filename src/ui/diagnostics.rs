//! GitHub #200: where a page request spent its time. Every `chat` request and every request that takes
//! `SLOW` or longer, and every timeout the page reports (`client_timeout`), is appended to `request-diagnostics.log` next to the workspace registry; the log is
//! bounded and a write failure never changes the answer to the page.
use std::{fs, io::Write, path::PathBuf, time::Duration};

const SLOW: Duration = Duration::from_secs(1);
const LIMIT: u64 = 256 * 1024;

fn path() -> Option<PathBuf> {
    Some(
        crate::desktop::workspace()
            .ok()?
            .parent()?
            .join("request-diagnostics.log"),
    )
}

pub(super) fn kind(bytes: &[u8]) -> String {
    #[derive(serde::Deserialize)]
    struct Kind {
        kind: String,
    }
    serde_json::from_slice::<Kind>(bytes).map_or_else(|_| "unreadable".into(), |k| k.kind)
}

fn append(line: &str) {
    let Some(path) = path() else { return };
    let _ = path.parent().map(fs::create_dir_all);
    if fs::metadata(&path).is_ok_and(|m| m.len() > LIMIT) {
        let _ = fs::rename(&path, path.with_extension("log.1"));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| writeln!(file, "{now} {line}"));
}

/// GitHub #221: the page gave up waiting for a reply to `of` after `waited` ms of its own clock. `waited` far
/// above the page's limit, or `hidden`, points at a suspended page rather than a slow request.
pub(super) fn client_timeout(report: &serde_json::Value) {
    let of: String = report["of"]
        .as_str()
        .unwrap_or("unknown")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(40)
        .collect();
    append(&format!(
        "client-timeout of={of} waited={} hidden={}",
        report["waited"].as_u64().unwrap_or(0),
        report["hidden"].as_bool().unwrap_or(false)
    ));
}

/// A named step of a request that took long enough to matter (`lock` is the wait for the settings lock).
pub(super) fn step(name: &str, elapsed: Duration) {
    if let Some(line) = crate::request_steps::line(name, elapsed) {
        append(&line);
    }
}

/// The request reached the native side; only chat requests are worth a line before they finish.
pub(super) fn start(kind: &str, size: usize) {
    if kind == "chat" {
        append(&format!("start kind={kind} bytes={size}"));
    }
}

pub(super) fn done(kind: &str, elapsed: Duration, error: Option<&String>) {
    if kind != "chat" && elapsed < SLOW {
        return;
    }
    let reason = error.map_or(String::new(), |e| {
        format!(" error={:?}", e.chars().take(120).collect::<String>())
    });
    append(&format!(
        "done kind={kind} ms={} ok={}{reason}",
        elapsed.as_millis(),
        error.is_none()
    ));
}
