use super::{MAX_RESPONSE, Server, config};
use crate::process::ProcessGroup;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};
struct Local {
    child: Child,
    group: ProcessGroup,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    stderr: tokio::task::JoinHandle<()>,
}
enum Transport {
    /// Boxed: a local child process with pipes dwarfs the remote variant (Windows clippy).
    Local(Box<Local>),
    Remote {
        url: String,
        headers: BTreeMap<String, String>,
        session: Option<String>,
    },
}
pub(super) struct Client {
    transport: Transport,
    timeout: u64,
    id: u64,
    bytes: usize,
}
impl Client {
    pub async fn open(server: &Server, cwd: &Path) -> Result<Self, String> {
        if !server.enabled() {
            return Err("MCP server is disabled".into());
        }
        let transport = match server {
            Server::Local {
                command,
                environment,
                cwd: dir,
                ..
            } => {
                let mut c = Command::new(&command[0]);
                c.args(&command[1..])
                    .current_dir(
                        dir.as_ref()
                            .map(|d| cwd.join(d))
                            .unwrap_or_else(|| cwd.to_owned()),
                    )
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true);
                for (k, v) in environment {
                    c.env(k, config::expand(v)?);
                }
                ProcessGroup::prepare(&mut c);
                let mut child = c
                    .spawn()
                    .map_err(|_| "could not start configured MCP command")?;
                let group = ProcessGroup::adopt(&child)?;
                let input = child.stdin.take().ok_or("MCP stdin missing")?;
                let output = BufReader::new(child.stdout.take().ok_or("MCP stdout missing")?);
                let mut errors = child.stderr.take().ok_or("MCP stderr missing")?;
                // Drain without retaining server output, which may contain credentials.
                let stderr = tokio::spawn(async move {
                    let mut b = [0; 4096];
                    while matches!(errors.read(&mut b).await,Ok(n)if n>0) {}
                });
                Transport::Local(Box::new(Local {
                    child,
                    group,
                    input,
                    output,
                    stderr,
                }))
            }
            Server::Remote { url, headers, .. } => Transport::Remote {
                url: url.clone(),
                headers: headers
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), config::expand(v)?)))
                    .collect::<Result<_, String>>()?,
                session: None,
            },
        };
        let mut c = Self {
            transport,
            timeout: server.timeout(),
            id: 0,
            bytes: 0,
        };
        let init=c.rpc("initialize",json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"AgentConductor","version":env!("CARGO_PKG_VERSION")}})).await;
        match init {
            Ok(v)
                if ["2025-06-18", "2025-03-26", "2024-11-05"]
                    .contains(&v["protocolVersion"].as_str().unwrap_or("")) => {}
            Ok(_) => {
                c.close().await;
                return Err("MCP protocol version unsupported".into());
            }
            Err(e) => {
                c.close().await;
                return Err(e);
            }
        }
        if let Err(e) = c
            .send(
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                None,
            )
            .await
        {
            c.close().await;
            return Err(e);
        }
        Ok(c)
    }
    pub async fn rpc(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.id += 1;
        self.send(
            json!({"jsonrpc":"2.0","id":self.id,"method":method,"params":params}),
            Some(self.id),
        )
        .await?
        .ok_or_else(|| "MCP response missing".into())
    }
    async fn send(&mut self, packet: Value, id: Option<u64>) -> Result<Option<Value>, String> {
        let deadline = Duration::from_millis(self.timeout);
        tokio::time::timeout(deadline, self.exchange(packet, id))
            .await
            .map_err(|_| "MCP request timed out".to_string())?
    }
    async fn exchange(&mut self, packet: Value, id: Option<u64>) -> Result<Option<Value>, String> {
        let response = match &mut self.transport {
            Transport::Local(l) => {
                let mut bytes = serde_json::to_vec(&packet).map_err(|_| "invalid MCP request")?;
                bytes.push(b'\n');
                l.input
                    .write_all(&bytes)
                    .await
                    .map_err(|_| "MCP stdin closed")?;
                l.input.flush().await.map_err(|_| "MCP stdin closed")?;
                if id.is_none() {
                    return Ok(None);
                }
                let mut found = None;
                for _ in 0..64 {
                    let mut line = Vec::new();
                    loop {
                        let available =
                            l.output.fill_buf().await.map_err(|_| "MCP stdout failed")?;
                        if available.is_empty() {
                            return Err("MCP stdout closed before response".into());
                        }
                        let n = available
                            .iter()
                            .position(|b| *b == b'\n')
                            .map_or(available.len(), |p| p + 1);
                        if line.len() + n > MAX_RESPONSE {
                            return Err("MCP response exceeds 256 KiB".into());
                        }
                        line.extend_from_slice(&available[..n]);
                        l.output.consume(n);
                        if line.last() == Some(&b'\n') {
                            break;
                        }
                    }
                    self.bytes += line.len();
                    if self.bytes > 2 * 1024 * 1024 {
                        return Err("MCP session output exceeds 2 MiB".into());
                    }
                    let v: Value =
                        serde_json::from_slice(&line).map_err(|_| "MCP emitted malformed JSONL")?;
                    if v["id"] == json!(id) {
                        found = Some(v);
                        break;
                    }
                    // Never respond to server-initiated requests (sampling, elicitation, roots).
                    if v.get("id").is_some() {
                        return Err("unexpected MCP request/response ID".into());
                    }
                }
                found.ok_or("too many MCP notifications")?
            }
            Transport::Remote {
                url,
                headers,
                session,
            } => {
                let (v, new_session) = super::http::request(
                    url,
                    headers,
                    session.as_deref(),
                    &packet,
                    self.timeout,
                    "POST",
                )
                .await?;
                if let Some(s) = new_session {
                    if s.len() > 256 || s.contains(['\r', '\n']) {
                        return Err("invalid MCP session ID".into());
                    }
                    *session = Some(s);
                }
                if id.is_none() {
                    return Ok(None);
                }
                let v = v.ok_or("MCP response missing")?;
                self.bytes += v.to_string().len();
                if self.bytes > 2 * 1024 * 1024 {
                    return Err("MCP session output exceeds 2 MiB".into());
                }
                v
            }
        };
        if response["jsonrpc"] != "2.0" || response["id"] != json!(id) {
            return Err("invalid MCP response identity".into());
        }
        if response.get("error").is_some() {
            return Err("MCP returned a JSON-RPC error".into());
        }
        response
            .get("result")
            .cloned()
            .map(Some)
            .ok_or_else(|| "MCP result missing".into())
    }
    pub async fn close(&mut self) {
        if let Transport::Local(l) = &mut self.transport {
            l.group.kill();
            let _ = l.child.wait().await;
            l.stderr.abort();
        } else if let Transport::Remote {
            url,
            headers,
            session: Some(session),
        } = &self.transport
        {
            let _ = tokio::time::timeout(
                Duration::from_millis(800),
                super::http::request(url, headers, Some(session), &Value::Null, 750, "DELETE"),
            )
            .await;
        }
    }
}
