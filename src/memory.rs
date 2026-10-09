use crate::state::Store;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

/// Optional delivery to an existing loopback agentmemory daemon. No model API involved.
pub fn sync(store: &Store, port: u16) -> Result<usize, String> {
    if port == 0 {
        return Err("memory port must be nonzero".into());
    }
    let mut count = 0;
    for (id, content) in store.pending_memory()? {
        let body = serde_json::to_vec(&serde_json::json!({
            "content":format!("AgentConductor outbox item {id}: {content}"),
            "concepts":["agentconductor","workflow-progress"]
        }))
        .map_err(|e| e.to_string())?;
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .map_err(|e| format!("local agentmemory unavailable: {e}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| e.to_string())?;
        let auth = match std::env::var("AGENTMEMORY_SECRET") {
            Ok(secret) if secret.contains(['\r', '\n']) => {
                return Err("invalid memory authorization value".into());
            }
            Ok(secret) => format!("Authorization: Bearer {secret}\r\n"),
            Err(_) => String::new(),
        };
        let header = format!(
            "POST /agentmemory/remember HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth}Connection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(header.as_bytes())
            .and_then(|_| stream.write_all(&body))
            .map_err(|e| e.to_string())?;
        let mut response = vec![];
        let mut bytes = [0; 1024];
        loop {
            let n = stream.read(&mut bytes).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("memory server closed without response headers".into());
            }
            response.extend_from_slice(&bytes[..n]);
            if response.len() > 16 * 1024 {
                return Err("memory response headers too large".into());
            }
            if response.windows(4).any(|p| p == b"\r\n\r\n") {
                break;
            }
        }
        let first = String::from_utf8_lossy(&response)
            .lines()
            .next()
            .unwrap_or_default()
            .to_string();
        if !(first.starts_with("HTTP/1.1 201 ") || first.starts_with("HTTP/1.0 201 ")) {
            return Err("memory server did not confirm creation; outbox retained".into());
        }
        store.memory_delivered(id)?;
        count += 1;
    }
    Ok(count)
}
