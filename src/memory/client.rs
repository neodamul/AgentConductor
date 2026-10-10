use super::checkpoints::{Envelope, Receipt};
use crate::workflow;
use serde::Deserialize;
use serde_json::Value;
use std::{
    io::{Read, Write},
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
#[derive(Clone, Copy)]
enum Framing {
    Length(usize),
    Chunked,
}
fn headers(bytes: &[u8]) -> Result<(u16, Framing), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid memory response headers")?;
    let mut lines = text.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    if !matches!(first.next(), Some("HTTP/1.1" | "HTTP/1.0")) {
        return Err("invalid memory response protocol".into());
    }
    let status = first
        .next()
        .ok_or("missing memory response status")?
        .parse::<u16>()
        .map_err(|_| "invalid memory response status")?;
    if !(100..=599).contains(&status) {
        return Err("invalid memory response status".into());
    }
    let mut length = None;
    let mut transfer = false;
    for line in lines {
        let (key, value) = line
            .split_once(':')
            .ok_or("malformed memory response header")?;
        let value = value.trim();
        if key.eq_ignore_ascii_case("content-length") {
            let n = value
                .parse::<usize>()
                .map_err(|_| "invalid memory content length")?;
            if n > BODY_MAX || length.replace(n).is_some() {
                return Err("oversized or duplicate memory content length".into());
            }
        } else if key.eq_ignore_ascii_case("transfer-encoding") {
            if transfer || !value.eq_ignore_ascii_case("chunked") {
                return Err("unsupported memory transfer encoding".into());
            }
            transfer = true;
        } else if key.eq_ignore_ascii_case("content-encoding")
            && !value.eq_ignore_ascii_case("identity")
        {
            return Err("unsupported memory content encoding".into());
        }
    }
    match (length, transfer) {
        (Some(n), false) => Ok((status, Framing::Length(n))),
        (None, true) => Ok((status, Framing::Chunked)),
        _ => Err("ambiguous or missing memory response framing".into()),
    }
}
fn read_more(
    stream: &mut TcpStream,
    data: &mut Vec<u8>,
    buf: &mut [u8],
    started: Instant,
    deadline: Duration,
) -> Result<bool, String> {
    let left = deadline
        .checked_sub(started.elapsed())
        .filter(|d| !d.is_zero())
        .ok_or("memory response deadline exceeded")?;
    stream
        .set_read_timeout(Some(left))
        .map_err(|e| e.to_string())?;
    let n = stream
        .read(buf)
        .map_err(|e| format!("memory response unavailable: {e}"))?;
    if data.len() + n > WIRE_MAX {
        return Err("memory response exceeds wire limit".into());
    }
    data.extend_from_slice(&buf[..n]);
    Ok(n == 0)
}
fn chunked(data: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut cursor = 0;
    let mut body = vec![];
    for _ in 0..1024 {
        let Some(offset) = data[cursor..].windows(2).position(|s| s == b"\r\n") else {
            return Ok(None);
        };
        if offset > 128 {
            return Err("memory chunk header exceeds limit".into());
        }
        let text = std::str::from_utf8(&data[cursor..cursor + offset])
            .map_err(|_| "invalid memory chunk header")?;
        let count = usize::from_str_radix(text.split(';').next().unwrap_or_default(), 16)
            .map_err(|_| "invalid memory chunk length")?;
        cursor += offset + 2;
        if count == 0 {
            if data.len() < cursor + 2 {
                return Ok(None);
            }
            if &data[cursor..cursor + 2] != b"\r\n" {
                return Err("memory chunk trailers unsupported".into());
            }
            return Ok(Some(body));
        }
        if count > BODY_MAX - body.len() {
            return Err("memory chunked body exceeds limit".into());
        }
        if data.len() < cursor + count + 2 {
            return Ok(None);
        }
        body.extend_from_slice(&data[cursor..cursor + count]);
        cursor += count;
        if &data[cursor..cursor + 2] != b"\r\n" {
            return Err("invalid memory chunk terminator".into());
        }
        cursor += 2;
    }
    Err("too many memory response chunks".into())
}

fn bounded_file(path: &std::path::Path, max: u64) -> Result<Option<String>, String> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err("cannot read existing agentmemory authentication configuration".into());
        }
    };
    let mut value = String::new();
    file.take(max + 1)
        .read_to_string(&mut value)
        .map_err(|_| "invalid agentmemory authentication configuration")?;
    if value.len() as u64 > max {
        return Err("agentmemory authentication configuration exceeds limit".into());
    }
    Ok(Some(value))
}
fn load_secret(home: &std::path::Path) -> Result<Option<String>, String> {
    if let Some(content) = bounded_file(&home.join(".env"), 64 * 1024)? {
        for line in content.lines().rev() {
            let line = line.trim().strip_prefix("export ").unwrap_or(line.trim());
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if key.trim() != "AGENTMEMORY_SECRET" {
                continue;
            }
            let value = value.trim();
            let value = if value.starts_with(['\"', '\'']) {
                let quote = value.chars().next().unwrap();
                value[1..]
                    .split_once(quote)
                    .ok_or("invalid agentmemory secret quoting")?
                    .0
            } else {
                value.split(" #").next().unwrap_or_default().trim()
            };
            if !value.is_empty() && !value.starts_with("${") {
                return Ok(Some(value.into()));
            }
            break;
        }
    }
    Ok(bounded_file(&home.join("secret"), 4096)?
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty()))
}

#[cfg(test)]
#[path = "../../tests/unit/memory_client.rs"]
mod tests;
