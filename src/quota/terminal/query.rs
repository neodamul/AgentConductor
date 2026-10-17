use super::*;

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
    let collect = async {
        let mut data = Vec::new();
        let mut found = None;
        // Output before the last trust answer is not read again.
        let (mut start, mut answers) = (0, 0);
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
                if let Ok(snapshot) = parse_claude_usage(&text, now) {
                    found = Some(snapshot);
                }
            }
        }
    };
    let result = tokio::time::timeout(Duration::from_secs(20), collect).await;
    owner.timed_out = result.is_err();
    let result = result
        .map_err(|_| quota_timeout(sent).to_string())
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
