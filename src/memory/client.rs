use super::checkpoints::{Envelope, Receipt};
use crate::workflow;
use serde::Deserialize;
use serde_json::Value;
use transport::{Framing, chunked, headers, load_secret, read_more};

#[path = "transport.rs"]
mod transport;
use std::{
    io::Write,
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
};
const HEADER_MAX: usize = 16 * 1024;
const BODY_MAX: usize = 64 * 1024;
const WIRE_MAX: usize = 128 * 1024;

/// Existing daemon client only. No listener, model endpoint, proxy, redirects or retries.
pub struct Client {
    port: u16,
    secret: Option<String>,
}
#[derive(Deserialize)]
struct Memory {
    id: String,
    project: String,
    #[serde(rename = "agentId")]
    agent_id: String,
    content: String,
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        && id != "."
        && id != ".."
}
impl Client {
    pub fn new(port: u16, secret: Option<String>) -> Result<Self, String> {
        if port == 0 {
            return Err("memory port must be nonzero".into());
        }
        if secret.as_ref().is_some_and(|s| {
            s.is_empty() || s.len() > 4096 || !s.bytes().all(|c| c.is_ascii_graphic())
        }) {
            return Err("invalid memory authorization value".into());
        }
        Ok(Self { port, secret })
    }
    /// Reuses existing local daemon credentials. It never registers a model API key.
    pub fn from_environment(port: u16) -> Result<Self, String> {
        Self::new(port, None)?;
        let secret = match std::env::var("AGENTMEMORY_SECRET") {
            Ok(value) if !value.trim().is_empty() => Some(value.trim().to_string()),
            _ => {
                let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
                home.map(|home| load_secret(&std::path::PathBuf::from(home).join(".agentmemory")))
                    .transpose()?
                    .flatten()
            }
        };
        Self::new(port, secret)
    }
    pub fn publish(&self, packet: &Envelope) -> Result<Receipt, String> {
        let (project, agent_id) = packet.scope()?;
        let content =
            serde_json::to_string(packet).map_err(|_| "checkpoint serialization failed")?;
        if content.len() > 16 * 1024 {
            return Err("checkpoint envelope exceeds 16 KiB".into());
        }
        let body = serde_json::json!({"content":content,"project":project,"agentId":agent_id,"type":"workflow","concepts":["agentconductor","cooperative-checkpoint"]});
        let response = self.request("POST", "/agentmemory/remember", Some(&body), 201)?;
        if response["success"] != true {
            return Err("memory creation was not confirmed".into());
        }
        let memory: Memory = serde_json::from_value(response["memory"].clone())
            .map_err(|_| "invalid memory creation receipt")?;
        if !valid_id(&memory.id)
            || memory.project != project
            || memory.agent_id != agent_id
            || memory.content != content
        {
            return Err("memory creation receipt does not match checkpoint".into());
        }
        Ok(Receipt {
            memory_id: memory.id,
            project,
            agent_id,
            content_hash: workflow::digest(content.as_bytes()),
        })
    }
    pub(super) fn remember_note(&self, content: &str) -> Result<String, String> {
        if content.len() > 32 * 1024 {
            return Err("memory outbox note exceeds 32 KiB".into());
        }
        let response=self.request("POST","/agentmemory/remember",Some(&serde_json::json!({"content":content,"project":"AgentConductor","type":"workflow","concepts":["agentconductor","workflow-progress"]})),201)?;
        let id = response["memory"]["id"]
            .as_str()
            .filter(|id| valid_id(id))
            .ok_or("invalid memory creation receipt")?;
        if response["success"] != true
            || response["memory"]["content"] != content
            || response["memory"]["project"] != "AgentConductor"
        {
            return Err("memory outbox creation receipt does not match observation".into());
        }
        Ok(id.to_string())
    }
    /// IDs of the latest memories carrying exactly this run project and agent tag.
    pub fn list_scope(&self, project: &str, agent_id: &str) -> Result<Vec<String>, String> {
        if !agent_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-' || c == b'_')
        {
            return Err("invalid memory agent tag".into());
        }
        let response = self.request(
            "GET",
            &format!("/agentmemory/memories?agentId={agent_id}&latest=true&limit=50"),
            None,
            200,
        )?;
        let entries = response["memories"]
            .as_array()
            .ok_or("invalid memory listing response")?;
        Ok(entries
            .iter()
            .filter_map(|entry| serde_json::from_value::<Memory>(entry.clone()).ok())
            .filter(|m| m.project == project && m.agent_id == agent_id && valid_id(&m.id))
            .map(|m| m.id)
            .collect())
    }
    pub fn read(&self, id: &str) -> Result<Envelope, String> {
        if !valid_id(id) {
            return Err("invalid memory ID".into());
        }
        let response = self.request("GET", &format!("/agentmemory/memories/{id}"), None, 200)?;
        let memory: Memory = serde_json::from_value(response["memory"].clone())
            .map_err(|_| "invalid exact memory response")?;
        if memory.id != id || memory.content.len() > 16 * 1024 {
            return Err("exact memory response identity or size mismatch".into());
        }
        let packet: Envelope = serde_json::from_str(&memory.content)
            .map_err(|_| "memory is not a typed checkpoint envelope")?;
        let (project, agent_id) = packet.scope()?;
        if memory.project != project || memory.agent_id != agent_id {
            return Err("memory checkpoint provenance mismatch".into());
        }
        Ok(packet)
    }
    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
        expected: u16,
    ) -> Result<Value, String> {
        let started = Instant::now();
        let deadline = Duration::from_secs(8);
        let mut stream = TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], self.port)),
            Duration::from_secs(2),
        )
        .map_err(|e| format!("local agentmemory unavailable: {e}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| e.to_string())?;
        let data = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|_| "memory request serialization failed")?
            .unwrap_or_default();
        if data.len() > BODY_MAX {
            return Err("memory request exceeds limit".into());
        }
        let auth = self
            .secret
            .as_ref()
            .map(|s| format!("Authorization: Bearer {s}\r\n"))
            .unwrap_or_default();
        let header = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nContent-Length: {}\r\n{auth}Connection: close\r\n\r\n",
            self.port,
            data.len()
        );
        stream
            .write_all(header.as_bytes())
            .and_then(|_| stream.write_all(&data))
            .map_err(|e| e.to_string())?;
        let mut response = vec![];
        let mut buf = [0; 4096];
        let (status, framing, end) = loop {
            if read_more(&mut stream, &mut response, &mut buf, started, deadline)? {
                return Err("memory server closed before response headers".into());
            }
            if let Some(end) = response.windows(4).position(|s| s == b"\r\n\r\n") {
                if end + 4 > HEADER_MAX {
                    return Err("memory response headers exceed limit".into());
                }
                let (status, framing) = headers(&response[..end])?;
                break (status, framing, end + 4);
            }
            if response.len() > HEADER_MAX {
                return Err("memory response headers exceed limit".into());
            }
        };
        if status != expected {
            return Err(format!(
                "agentmemory returned HTTP {status}; no checkpoint accepted"
            ));
        }
        let body = loop {
            let data = &response[end..];
            match framing {
                Framing::Length(n) if data.len() >= n => break data[..n].to_vec(),
                Framing::Chunked => {
                    if let Some(body) = chunked(data)? {
                        break body;
                    }
                }
                _ => {}
            }
            let eof = read_more(&mut stream, &mut response, &mut buf, started, deadline)?;
            if eof {
                return Err("truncated memory response".into());
            }
        };
        serde_json::from_slice(&body).map_err(|_| "agentmemory returned invalid JSON".into())
    }
}
#[cfg(test)]
#[path = "../../tests/unit/memory_client.rs"]
mod tests;
