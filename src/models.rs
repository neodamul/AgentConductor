use crate::{
    plan::{AgentProfile, CatalogOrigin, ModelCatalog},
    protocol::Provider,
};
use serde_json::{Value, json};
use std::{
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

mod custom;
pub use custom::parse_custom_catalog;

pub fn parse_catalog(provider: Provider, response: &Value) -> Result<Vec<String>, String> {
    let entries = match provider {
        Provider::Claude => response.pointer("/response/response/models"),
        Provider::Codex => response.pointer("/result/data"),
        _ => return Err("catalog JSON parsing is unsupported for this provider".into()),
    }
    .and_then(Value::as_array)
    .ok_or("CLI did not return a supported model catalog; import an operator-observed catalog")?;
    let mut models = vec![];
    for entry in entries {
        if entry.get("hidden").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let model = entry
            .get(if provider == Provider::Claude {
                "value"
            } else {
                "model"
            })
            .and_then(Value::as_str)
            .ok_or("catalog model ID missing")?;
        if model.is_empty() || model.starts_with('-') || model.chars().any(char::is_whitespace) {
            return Err("invalid catalog model ID".into());
        }
        if !models.iter().any(|m| m == model) {
            models.push(model.to_string());
        }
    }
    if models.is_empty() {
        return Err("CLI returned an empty model catalog".into());
    }
    Ok(models)
}

pub async fn discover(profile: &AgentProfile, dir: &Path) -> Result<ModelCatalog, String> {
    use crate::process::{self, ProcessGroup, ProcessSpec};
    use std::process::Stdio;
    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
        process::Command,
    };
    if profile.parser == Provider::Custom {
        return custom::discover(profile, dir).await;
    }
    tokio::fs::create_dir(dir)
        .await
        .map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    let version = process::run(ProcessSpec {
        executable: profile.executable.clone().into(),
        args: vec!["--version".into()],
        cwd: dir.clone(),
        input: vec![],
        log_dir: dir.join("version"),
        timeout: Duration::from_secs(15),
        signal_grace: Duration::from_secs(1),
        environment: profile.environment.clone(),
    })
    .await?;
    if version.exit_code != Some(0) || version.timed_out {
        return Err("CLI version query failed; inspect discovery logs".into());
    }
    let cli_version = std::fs::read_to_string(version.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .chars()
        .take(256)
        .collect::<String>();
    if profile.parser == Provider::OpenCode {
        let result = process::run(ProcessSpec {
            executable: profile.executable.clone().into(),
            args: vec!["models".into()],
            cwd: dir.clone(),
            input: vec![],
            log_dir: dir.join("query"),
            timeout: Duration::from_secs(30),
            signal_grace: Duration::from_secs(1),
            environment: profile.environment.clone(),
        })
        .await?;
        if result.exit_code != Some(0) || result.timed_out {
            return Err("OpenCode model query failed; inspect discovery logs".into());
        }
        let text = std::fs::read_to_string(result.stdout).map_err(|e| e.to_string())?;
        let models: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        if models.is_empty()
            || models.iter().any(|m| {
                !m.contains('/') || m.starts_with('-') || m.chars().any(char::is_whitespace)
            })
        {
            return Err("invalid OpenCode model listing; import an observed catalog".into());
        }
        return Ok(ModelCatalog {models,source:"opencode models (configured-provider catalog; entitlement is verified during execution)".into(),queried_at:timestamp(),cli_version,origin:CatalogOrigin::Discovered});
    }
    let args = if profile.parser == Provider::Codex {
        vec!["app-server"]
    } else {
        vec![
            "-p",
            "--output-format",
            "stream-json",
            "--input-format",
            "stream-json",
            "--verbose",
            "--tools",
            "",
            "--strict-mcp-config",
            "--mcp-config",
            "{\"mcpServers\":{}}",
        ]
    };
    let stderr = std::fs::File::create(dir.join("query-stderr.log")).map_err(|e| e.to_string())?;
    let executable = crate::executable::resolve(&profile.executable)
        .unwrap_or_else(|| profile.executable.clone().into());
    let mut command = Command::new(executable);
    command
        .args(args)
        .envs(&profile.environment)
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .kill_on_drop(true);
    ProcessGroup::prepare(&mut command);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let group = ProcessGroup::adopt(&child)?;
    let mut input = child.stdin.take().ok_or("discovery stdin missing")?;
    let mut reader = BufReader::new(child.stdout.take().ok_or("discovery stdout missing")?);
    let mut evidence = tokio::fs::File::create(dir.join("query-stdout.jsonl"))
        .await
        .map_err(|e| e.to_string())?;
    async fn send(input: &mut tokio::process::ChildStdin, value: Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        input.write_all(&bytes).await.map_err(|e| e.to_string())
    }
    let query = async {
        let claude = profile.parser == Provider::Claude;
        send(&mut input,if claude {json!({"type":"control_request","request_id":"catalog-init","request":{"subtype":"initialize"}})} else {json!({"method":"initialize","id":1,"params":{"clientInfo":{"name":"agentconductor","title":"AgentConductor","version":env!("CARGO_PKG_VERSION")}}})}).await?;
        let mut models = vec![];
        let mut request_id = 2u64;
        loop {
            let mut line = vec![];
            let n = (&mut reader)
                .take(1024 * 1024 + 1)
                .read_until(b'\n', &mut line)
                .await
                .map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("CLI closed before returning a model catalog".into());
            }
            if n > 1024 * 1024 {
                return Err("discovery event exceeded limit".into());
            }
            evidence.write_all(&line).await.map_err(|e| e.to_string())?;
            let message: Value = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
            if claude
                && message
                    .pointer("/response/request_id")
                    .and_then(Value::as_str)
                    == Some("catalog-init")
            {
                return parse_catalog(profile.parser, &message);
            }
            if !claude && message["id"] == 1 {
                if message.get("error").is_some() {
                    return Err("Codex initialize failed; inspect discovery logs".into());
                }
                send(&mut input, json!({"method":"initialized","params":{}})).await?;
                send(&mut input,json!({"method":"model/list","id":request_id,"params":{"limit":100,"includeHidden":false}})).await?;
            } else if !claude && message["id"] == request_id {
                models.extend(parse_catalog(profile.parser, &message)?);
                if let Some(cursor) = message
                    .pointer("/result/nextCursor")
                    .and_then(Value::as_str)
                {
                    request_id += 1;
                    send(&mut input,json!({"method":"model/list","id":request_id,"params":{"limit":100,"includeHidden":false,"cursor":cursor}})).await?;
                } else {
                    models.sort();
                    models.dedup();
                    return Ok(models);
                }
            }
        }
    };
    let result = tokio::time::timeout(Duration::from_secs(30), query)
        .await
        .map_err(|_| "model discovery timed out; no model generation was requested".to_string())
        .and_then(|r| r);
    drop(input);
    drop(group);
    let _ = child.wait().await;
    let models = result?;
    Ok(ModelCatalog {
        models,
        source: if profile.parser == Provider::Codex {
            "codex app-server stdio model/list"
        } else {
            "claude stream-json control initialization"
        }
        .into(),
        queried_at: timestamp(),
        cli_version,
        origin: CatalogOrigin::Discovered,
    })
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
