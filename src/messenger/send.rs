//! DD-79 delivery through the system `curl`, without a shell. URL and body go through
//! `--config -` on stdin, so the secret never appears in a process list or log.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread::JoinHandle;

static PENDING: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());
/// One delivery at a time keeps the cost bounded and the order of messages stable.
static ORDER: Mutex<()> = Mutex::new(());

fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Posts `body` (a JSON object) and waits for the HTTP result (10 s per attempt, 2 retries).
pub fn deliver(url: &str, body: &str) -> Result<(), String> {
    super::validate_url(url)?;
    // DD-111: readable paths; `body` is JSON, so a `\\?\` prefix appears escaped.
    let body = &body
        .replace(r"\\\\?\\UNC\\", r"\\\\")
        .replace(r"\\\\?\\", "")
        .replace("//?/", "");
    let config = format!(
        "url = {}\nheader = \"Content-Type: application/json\"\ndata-binary = {}\n",
        quote(url),
        quote(body)
    );
    curl(url, &config).map(|_| ())
}

/// DD-81: GET with secret headers (for example a Slack bot token); returns at most 1 MiB.
pub fn fetch(url: &str, headers: &[String]) -> Result<String, String> {
    super::validate_url(url)?;
    let mut config = format!("url = {}\n", quote(url));
    for header in headers {
        config.push_str(&format!("header = {}\n", quote(header)));
    }
    let body = curl(url, &config)?;
    if body.len() > 1024 * 1024 {
        return Err("messenger response exceeds 1 MiB".into());
    }
    String::from_utf8(body).map_err(|_| "messenger response is not UTF-8".into())
}

/// Runs the system curl with `config` on stdin; the secret never reaches argv or logs.
fn curl(url: &str, config: &str) -> Result<Vec<u8>, String> {
    let _turn = ORDER.lock().unwrap_or_else(|e| e.into_inner());
    let program =
        std::env::var_os("AGENTCONDUCTOR_MESSENGER_CURL").unwrap_or_else(|| "curl".into());
    let mut child = crate::subprocess::command(program)
        .args([
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "10",
            "--retry",
            "2",
            "--config",
            "-",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot start curl: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("curl stdin unavailable")?
        .write_all(config.as_bytes())
        .map_err(|e| e.to_string())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let detail = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "messenger request failed: {}",
            detail.trim().replace(url, "<webhook>")
        ))
    }
}

pub(super) fn spawn(url: String, body: String, event: String, log: Option<PathBuf>) {
    let handle = std::thread::spawn(move || {
        if let Err(error) = deliver(&url, &body) {
            self::log(log.as_deref(), &event, &error);
        }
    });
    let mut pending = PENDING.lock().unwrap_or_else(|e| e.into_inner());
    pending.retain(|h| !h.is_finished());
    pending.push(handle);
}

pub(super) fn log(path: Option<&Path>, event: &str, error: &str) {
    if let Some(path) = path
        && let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
    {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        let _ = writeln!(file, "{now} {event}: {error}");
    }
}

/// Waits (bounded) for queued messages before a CLI process exits.
pub fn flush(limit: std::time::Duration) {
    let deadline = std::time::Instant::now() + limit;
    let handles: Vec<_> = std::mem::take(&mut *PENDING.lock().unwrap_or_else(|e| e.into_inner()));
    for handle in handles {
        while !handle.is_finished() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        if handle.is_finished() {
            let _ = handle.join();
        }
    }
}
