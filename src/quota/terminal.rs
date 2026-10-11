use super::{Snapshot, snapshot, window};
use serde_json::json;

fn plain(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 27 {
            i += 1;
            if i >= bytes.len() {
                break;
            }
            if bytes[i] == b'[' {
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
            } else if bytes[i] == b']' {
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 7 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == 27 && bytes.get(i + 1) == Some(&b'\\') {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            } else {
                while i < bytes.len() && (0x20..=0x2f).contains(&bytes[i]) {
                    i += 1;
                }
                i += 1;
            }
        } else {
            if bytes[i] >= 32 || bytes[i] == b'\n' {
                out.push(bytes[i]);
            }
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Version-sensitive screen-reader metadata; unrelated statistics never count as capacity.
pub fn parse_claude_usage(text: &str, observed_at: i64) -> Result<Snapshot, String> {
    let text = plain(text);
    if text.contains("Permission Required:") {
        return Err("Claude project trust approval is required; no approval was sent".into());
    }
    let enterprise = text.lines().take(12).any(|line| {
        let line = line.to_ascii_lowercase();
        line.contains("enterprise plan") || line.contains("claude enterprise")
    });
    let mut pending = None;
    let mut windows = std::collections::BTreeMap::new();
    for line in text.lines().map(str::trim) {
        if line == "Current session" {
            pending = Some(("five_hour", 300));
            continue;
        }
        if line.starts_with("Current week") {
            let lower = line.to_ascii_lowercase();
            pending = Some((
                if lower.contains("opus") {
                    "seven_day_opus"
                } else if lower.contains("sonnet") {
                    "seven_day_sonnet"
                } else {
                    "seven_day"
                },
                10080,
            ));
            continue;
        }
        if line.starts_with("What's contributing") || line == "Usage credits" || line == "Session" {
            pending = None;
        }
        if let Some((label, minutes)) = pending
            && line.contains("used")
            && let Some((prefix, _)) = line.split_once('%')
        {
            let used = prefix
                .trim()
                .parse::<f64>()
                .map_err(|_| "unrecognized Claude quota percentage")?;
            if !used.is_finite() || !(0.0..=100.0).contains(&used) {
                return Err("invalid Claude quota percentage".into());
            }
            windows.insert(
                label,
                window(
                    "claude",
                    label,
                    &json!({"used_percentage":used}),
                    true,
                    Some(minutes),
                )?,
            );
            pending = None;
        }
    }
    if windows.is_empty() {
        return Err("Claude /usage did not report recognized capacity windows".into());
    }
    let mut result = snapshot(
        "claude /usage screen-reader metadata",
        observed_at,
        windows.into_values().collect(),
    )?;
    if enterprise {
        result.plan_type = Some("enterprise".into());
    }
    Ok(result)
}

#[cfg(not(any(target_os = "macos", windows)))]
pub async fn query_claude(
    _profile: &crate::plan::AgentProfile,
    _dir: &std::path::Path,
    _cwd: &std::path::Path,
) -> Result<Snapshot, String> {
    Err("Claude interactive quota collection is verified on macOS only; use owned stream metadata on this platform".into())
}

#[cfg(target_os = "macos")]
pub async fn query_claude(
    profile: &crate::plan::AgentProfile,
    dir: &std::path::Path,
    cwd: &std::path::Path,
) -> Result<Snapshot, String> {
    use std::os::unix::fs::PermissionsExt;
    use std::{
        process::Stdio,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        process::Command,
    };
    tokio::fs::create_dir(dir)
        .await
        .map_err(|e| e.to_string())?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
        .map_err(|e| e.to_string())?;
    let stderr = std::fs::File::create(dir.join("stderr.log")).map_err(|e| e.to_string())?;
    let mut command = Command::new("/usr/bin/script");
    command
        .args([
            "-q",
            "/dev/null",
            &profile.executable,
            "--ax-screen-reader",
            "--tools",
            "",
            "--strict-mcp-config",
            "--setting-sources",
            "",
            "--settings",
            "{\"disableAllHooks\":true}",
        ])
        .envs(&profile.environment)
        .env_remove("ANTHROPIC_API_KEY")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .kill_on_drop(true);
    crate::process::ProcessGroup::prepare(&mut command);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let pid = child.id().ok_or("quota process PID missing")?;
    let group = crate::process::ProcessGroup::adopt(&child)?;
    let mut owner = crate::process::Ownership {
        coordinator_pid: std::process::id(),
        pid: Some(pid),
        process_group: Some(pid as i32),
        phase: "running".into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    };
    crate::process::record_owner(dir, &owner)?;
    let mut input = child.stdin.take().ok_or("quota stdin missing")?;
    let mut output = child.stdout.take().ok_or("quota stdout missing")?;
    let mut spool = tokio::fs::File::create(dir.join("stdout.terminal"))
        .await
        .map_err(|e| e.to_string())?;
    let collect = async {
        let mut data = Vec::new();
        let mut sent = false;
        let mut found = None;
        loop {
            let mut buffer = [0; 8192];
            let count = match tokio::time::timeout(Duration::from_secs(1), output.read(&mut buffer))
                .await
            {
                Ok(n) => n.map_err(|e| e.to_string())?,
                Err(_) if found.is_some() => {
                    return found.ok_or_else(|| "quota missing".to_string());
                }
                Err(_) => continue,
            };
            if count == 0 {
                return found.ok_or_else(|| "Claude closed before reporting quota".to_string());
            }
            data.extend_from_slice(&buffer[..count]);
            if data.len() > 1024 * 1024 {
                return Err("Claude quota output exceeds 1 MiB".into());
            }
            spool
                .write_all(&buffer[..count])
                .await
                .map_err(|e| e.to_string())?;
            let text = plain(&String::from_utf8_lossy(&data));
            if text.contains("Permission Required:") {
                return Err(
                    "Claude project trust approval is required; no approval was sent".into(),
                );
            }
            if !sent && text.contains("Claude Code v") && text.contains("$") {
                input
                    .write_all(b"/usage\r")
                    .await
                    .map_err(|e| e.to_string())?;
                input.flush().await.map_err(|e| e.to_string())?;
                sent = true;
            }
            if sent {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs() as i64;
                if let Ok(snapshot) = parse_claude_usage(&text, now) {
                    found = Some(snapshot);
                }
            }
        }
    };
    let result = tokio::time::timeout(Duration::from_secs(20), collect).await;
    owner.timed_out = result.is_err();
    let result = result
        .map_err(|_| "Claude quota query timed out; no generation requested".to_string())
        .and_then(|r| r);
    drop(input);
    drop(group);
    let status = tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .map_err(|_| "quota process reap timed out")?
        .map_err(|e| e.to_string())?;
    owner.phase = "terminal".into();
    owner.exit_code = status.code();
    owner.termination_signal = crate::process::exit_signal(&status);
    crate::process::record_owner(dir, &owner)?;
    result
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::query_claude;
