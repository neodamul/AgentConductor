use super::{Snapshot, parse_codex, query_custom};
use serde_json::Value;

pub async fn query(
    profile: &crate::plan::AgentProfile,
    dir: &std::path::Path,
) -> Result<Snapshot, String> {
    use serde_json::json;
    use std::{
        process::Stdio,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
        process::Command,
    };
    if profile.parser == crate::protocol::Provider::Custom {
        return query_custom(profile, dir).await;
    }
    if profile.parser != crate::protocol::Provider::Codex {
        return Err("this CLI has no verified headless quota query; quota is unavailable".into());
    }
    tokio::fs::create_dir(dir)
        .await
        .map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let stderr = std::fs::File::create(dir.join("stderr.log")).map_err(|e| e.to_string())?;
    let executable = crate::executable::resolve(&profile.executable)
        .unwrap_or_else(|| profile.executable.clone().into());
    let mut command = Command::new(executable);
    command
        .arg("app-server")
        .envs(&profile.environment)
        .env_remove("OPENAI_API_KEY")
        .env_remove("CODEX_API_KEY")
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .kill_on_drop(true);
    crate::process::ProcessGroup::prepare(&mut command);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let pid = child.id().ok_or("quota query PID missing")?;
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
    crate::process::record_owner(&dir, &owner)?;
    let mut input = child.stdin.take().ok_or("quota query stdin missing")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("quota query stdout missing")?);
    let mut spool = tokio::fs::File::create(dir.join("stdout.jsonl"))
        .await
        .map_err(|e| e.to_string())?;
    async fn send(input: &mut tokio::process::ChildStdin, value: Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        input.write_all(&bytes).await.map_err(|e| e.to_string())
    }
    let query = async {
        send(&mut input,json!({"method":"initialize","id":1,"params":{"clientInfo":{"name":"agentconductor","title":"AgentConductor","version":env!("CARGO_PKG_VERSION")}}})).await?;
        let mut initialized = false;
        let mut total = 0usize;
        for _ in 0..128 {
            let mut line = Vec::new();
            let count = (&mut output)
                .take(1024 * 1024 + 1)
                .read_until(b'\n', &mut line)
                .await
                .map_err(|e| e.to_string())?;
            total = total.saturating_add(count);
            if count == 0 {
                return Err("CLI closed before quota metadata was returned".into());
            }
            if total > 1024 * 1024 {
                return Err("quota metadata exceeds 1 MiB".into());
            }
            spool.write_all(&line).await.map_err(|e| e.to_string())?;
            let message: Value = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
            if message["id"] == 1 && !initialized {
                if message.get("error").is_some() {
                    return Err(
                        "CLI metadata initialization failed; inspect local query logs".into(),
                    );
                }
                initialized = true;
                send(&mut input, json!({"method":"initialized","params":{}})).await?;
                send(
                    &mut input,
                    json!({"method":"account/rateLimits/read","id":2}),
                )
                .await?;
            } else if message["id"] == 2 && initialized {
                if let Some(error) = message.get("error") {
                    let detail = error["message"].as_str().unwrap_or_default();
                    if detail.contains("chatgpt authentication required") {
                        return Err("Codex is signed in with an API key; usage windows are reported only for ChatGPT sign-in (run codex login and choose ChatGPT)".into());
                    }
                    return Err("account quota query failed; inspect local query logs".into());
                }
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs() as i64;
                return parse_codex(&message, now);
            }
        }
        Err("too many quota metadata events".into())
    };
    let result = tokio::time::timeout(Duration::from_secs(30), query).await;
    owner.timed_out = result.is_err();
    let result = result
        .map_err(|_| "quota query timed out; no generation requested".to_string())
        .and_then(|r| r);
    drop(input);
    drop(group);
    let status = tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .map_err(|_| "quota child reap timed out")?
        .map_err(|e| e.to_string())?;
    owner.phase = "terminal".into();
    owner.exit_code = status.code();
    owner.termination_signal = crate::process::exit_signal(&status);
    crate::process::record_owner(&dir, &owner)?;
    result
}
