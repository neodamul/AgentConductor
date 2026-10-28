//! GitHub #200: where a page request spent its time. Every `chat` request and every request that takes
//! `SLOW` or longer is appended to `request-diagnostics.log` next to the workspace registry; the log is
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
