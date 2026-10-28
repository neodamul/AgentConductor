//! Bounded native request timing evidence for coordinator-chat investigations.
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

fn append(line: &str) {
    let Some(path) = path() else { return };
    let _ = path.parent().map(fs::create_dir_all);
    if fs::metadata(&path).is_ok_and(|metadata| metadata.len() > LIMIT) {
        let _ = fs::rename(&path, path.with_extension("log.1"));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{now} {line}"));
}

pub(super) fn kind(bytes: &[u8]) -> String {
    #[derive(serde::Deserialize)]
    struct Kind {
        kind: String,
    }
    serde_json::from_slice::<Kind>(bytes).map_or_else(|_| "unreadable".into(), |value| value.kind)
}

pub(super) fn start(kind: &str, size: usize) {
    if kind == "chat" {
        append(&format!("start kind={kind} bytes={size}"));
    }
}

pub(super) fn done(kind: &str, elapsed: Duration, error: Option<&String>) {
    if kind != "chat" && elapsed < SLOW {
        return;
    }
    let reason = error.map_or(String::new(), |value| {
        format!(" error={:?}", value.chars().take(120).collect::<String>())
    });
    append(&format!(
        "done kind={kind} ms={} ok={}{reason}",
        elapsed.as_millis(),
        error.is_none()
    ));
}

pub(super) fn client_timeout(report: &serde_json::Value) {
    let of = report["of"]
        .as_str()
        .unwrap_or("unknown")
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .take(40)
        .collect::<String>();
    append(&format!(
        "client-timeout of={of} waited={} hidden={}",
        report["waited"].as_u64().unwrap_or(0),
        report["hidden"].as_bool().unwrap_or(false)
    ));
}
