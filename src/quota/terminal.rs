use super::Snapshot;

mod parse;
mod screens;
pub use parse::parse_claude_usage;
#[cfg_attr(not(any(target_os = "macos", windows)), allow(unused_imports))]
use screens::{TRUST_ANSWERS, TRUST_REFUSED, plain};
pub use screens::{claude_blocker, claude_trust_answer, claude_trust_prompt, claude_usage_ready};

/// GitHub #297 / #315: the whole Claude query (start, trust screen, prompt, `/usage` answer) is given up
/// after this long. A company PC with scanning, managed settings or a proxy needed more than 20 s and
/// then more than 60 s; a fast machine is unaffected because the query ends as soon as `/usage` is parsed.
/// Custom quota commands allow up to 300 s, so this stays inside that range.
pub const CLAUDE_WAIT: std::time::Duration = std::time::Duration::from_secs(180);

/// `unread` is the last reason a screen could not be parsed: a screen that was read but not
/// understood must not look like a CLI that never answered (GitHub #295).
fn quota_timeout(sent: bool, wait: std::time::Duration, unread: Option<&str>) -> String {
    let stage = if sent {
        "response timed out after /usage"
    } else {
        "prompt timed out before /usage"
    };
    let unread = unread.map_or(String::new(), |reason| {
        format!("; screen not understood: {reason}")
    });
    format!(
        "Claude quota {stage} after {} s{unread}; no generation requested",
        wait.as_secs()
    )
}

/// GitHub #568: a `/usage` screen that stays unreadable and silent is not waited for until the limit
/// (a signed-out CLI or one billed per API use shows no subscription windows); the screen is named at once.
const SETTLE_CAP_SECS: u64 = 20;

fn settled_unreadable(wait: std::time::Duration) -> std::time::Duration {
    (wait / 4).min(std::time::Duration::from_secs(SETTLE_CAP_SECS))
}

fn quota_unreadable(reason: &str) -> String {
    format!(
        "Claude quota screen not understood after /usage: {reason}; if this Claude CLI is signed out or billed per API use, run claude, sign in with /login to your subscription, then refresh; no generation requested"
    )
}

pub async fn query_claude(
    profile: &crate::plan::AgentProfile,
    dir: &std::path::Path,
    cwd: &std::path::Path,
) -> Result<Snapshot, String> {
    query_claude_within(profile, dir, cwd, CLAUDE_WAIT).await
}
/// Claude only answers /usage inside a project folder it already trusts.
#[cfg_attr(not(any(target_os = "macos", windows)), allow(dead_code))]
fn require_project(cwd: &std::path::Path) -> Result<(), String> {
    if cwd.as_os_str().is_empty() {
        return Err(
            "Choose a project folder for this workspace first; Claude reports usage only inside a trusted project folder"
                .into(),
        );
    }
    Ok(())
}

#[cfg(not(any(target_os = "macos", windows)))]
pub async fn query_claude_within(
    _profile: &crate::plan::AgentProfile,
    _dir: &std::path::Path,
    _cwd: &std::path::Path,
    _wait: std::time::Duration,
) -> Result<Snapshot, String> {
    Err("Claude interactive quota collection is verified on macOS only; use owned stream metadata on this platform".into())
}

#[cfg(target_os = "macos")]
pub async fn query_claude_within(
    profile: &crate::plan::AgentProfile,
    dir: &std::path::Path,
    cwd: &std::path::Path,
    wait: std::time::Duration,
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
    require_project(cwd)?;
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
    let mut sent = false;
    let mut unread: Option<String> = None;
    let collect = async {
        let mut data = Vec::new();
        let mut found = None;
        // Output before the last trust answer is not read again.
        let (mut start, mut answers) = (0, 0);
        let (mut quiet, settle) = (0u64, settled_unreadable(wait));
        loop {
            let mut buffer = [0; 8192];
            let count = match tokio::time::timeout(Duration::from_secs(1), output.read(&mut buffer))
                .await
            {
                Ok(n) => {
                    quiet = 0;
                    n.map_err(|e| e.to_string())?
                }
                Err(_) if found.is_some() => {
                    return found.ok_or_else(|| "quota missing".to_string());
                }
                Err(_) => {
                    quiet += 1;
                    if sent
                        && Duration::from_secs(quiet) >= settle
                        && let Some(reason) = &unread
                    {
                        return Err(quota_unreadable(reason));
                    }
                    continue;
                }
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
            let text = plain(&String::from_utf8_lossy(&data[start..]));
            if claude_trust_prompt(&text) {
                if answers >= TRUST_ANSWERS {
                    return Err(TRUST_REFUSED.into());
                }
                input
                    .write_all(claude_trust_answer(&text))
                    .await
                    .map_err(|e| e.to_string())?;
                input.flush().await.map_err(|e| e.to_string())?;
                answers += 1;
                start = data.len();
                continue;
            }
            if let Some(blocker) = claude_blocker(&text) {
                return Err(blocker.into());
            }
            if !sent && claude_usage_ready(&text) {
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
                match parse_claude_usage(&text, now) {
                    Ok(snapshot) => found = Some(snapshot),
                    Err(reason) => unread = Some(reason),
                }
            }
        }
    };
    let result = tokio::time::timeout(wait, collect).await;
    owner.timed_out = result.is_err();
    let result = result
        .map_err(|_| quota_timeout(sent, wait, unread.as_deref()))
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
pub use windows::query_claude_within;
