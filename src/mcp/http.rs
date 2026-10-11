//! Bounded Streamable HTTP using the host TLS client, without a Rust HTTP dependency.
use super::MAX_RESPONSE;
use crate::process::ProcessGroup;
use serde_json::Value;
use std::{collections::BTreeMap, process::Stdio};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};
fn quoted(s: &str) -> String {
    format!(
        "\"{}\"",
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    )
}
pub(super) async fn request(
    url: &str,
    headers: &BTreeMap<String, String>,
    session: Option<&str>,
    packet: &Value,
    timeout: u64,
    method: &str,
) -> Result<(Option<Value>, Option<String>), String> {
    let mut cfg = format!(
        "url = {}\nrequest = {}\nheader = \"Content-Type: application/json\"\nheader = \"Accept: application/json, text/event-stream\"\nheader = \"MCP-Protocol-Version: 2025-06-18\"\n",
        quoted(url),
        quoted(method)
    );
    if method == "POST" {
        cfg.push_str(&format!("data = {}\n", quoted(&packet.to_string())));
    }
    for (k, v) in headers {
        cfg.push_str(&format!("header = {}\n", quoted(&format!("{k}: {v}"))));
    }
    if let Some(s) = session {
        cfg.push_str(&format!(
            "header = {}\n",
            quoted(&format!("Mcp-Session-Id: {s}"))
        ));
    }
    let mut cmd = Command::new("curl");
    cmd.args([
        "-q",
        "--config",
        "-",
        "--silent",
        "--no-buffer",
        "--include",
        "--suppress-connect-headers",
        "--max-time",
        &(timeout as f64 / 1000.0).to_string(),
        "--max-filesize",
        &MAX_RESPONSE.to_string(),
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .kill_on_drop(true);
    ProcessGroup::prepare(&mut cmd);
    let mut child = cmd
        .spawn()
        .map_err(|_| "system curl is required for remote MCP")?;
    let group = ProcessGroup::adopt(&child)?;
    let result = async {
        let mut input = child.stdin.take().ok_or("HTTP stdin missing")?;
        input
            .write_all(cfg.as_bytes())
            .await
            .map_err(|_| "HTTP request failed")?;
        drop(input);
        let mut output = child.stdout.take().ok_or("HTTP stdout missing")?;
        let mut bytes = Vec::new();
        let mut chunk = [0; 4096];
        loop {
            let n = output
                .read(&mut chunk)
                .await
                .map_err(|_| "MCP HTTP read failed")?;
            if bytes.len() + n > MAX_RESPONSE + 8192 {
                return Err("MCP HTTP response too large".into());
            }
            bytes.extend_from_slice(&chunk[..n]);
            if let Some(v) = decode(&bytes, packet.get("id"), method == "DELETE")? {
                return Ok(v);
            }
            if n == 0 {
                return Err("MCP HTTP closed without a complete response".into());
            }
        }
    }
    .await;
    group.kill();
    let _ = child.wait().await;
    result
}
type Response = (Option<Value>, Option<String>);
fn decode(bytes: &[u8], id: Option<&Value>, deleting: bool) -> Result<Option<Response>, String> {
    let text = match std::str::from_utf8(bytes) {
        Ok(t) => t,
        Err(e) if e.error_len().is_none() => return Ok(None),
        Err(_) => return Err("MCP HTTP response is not UTF-8".into()),
    };
    let Some((head, body)) = text
        .split_once("\r\n\r\n")
        .or_else(|| text.split_once("\n\n"))
    else {
        return Ok(None);
    };
    if head.len() > 8192 {
        return Err("MCP HTTP headers too large".into());
    }
    let code = head
        .lines()
        .next()
        .and_then(|s| s.split_whitespace().nth(1))
        .unwrap_or("");
    if code == "100" {
        return decode(body.as_bytes(), id, deleting);
    }
    if ["401", "403"].contains(&code) {
        return Err("MCP authentication required; configure environment-backed headers or an authenticated local proxy".into());
    }
    if deleting {
        return if ["200", "202", "204", "405"].contains(&code) {
            Ok(Some((None, None)))
        } else {
            Err("MCP session close failed".into())
        };
    }
    if code != "200" && code != "202" {
        return Err("MCP HTTP status was not successful (redirects are refused)".into());
    }
    let sid = head.lines().find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.eq_ignore_ascii_case("mcp-session-id")
            .then(|| v.trim().to_owned())
    });
    if sid
        .as_ref()
        .is_some_and(|s| s.len() > 256 || s.contains(['\r', '\n']))
    {
        return Err("invalid MCP session ID".into());
    }
    if id.is_none() {
        return if code == "202" {
            Ok(Some((None, sid)))
        } else {
            Err("MCP notification was not accepted with 202".into())
        };
    }
    if head.to_ascii_lowercase().contains("text/event-stream") {
        let body = body.replace("\r\n", "\n");
        let mut events = body.split("\n\n").collect::<Vec<_>>();
        events.pop();
        for event in events {
            let data = event
                .lines()
                .filter_map(|l| l.strip_prefix("data:"))
                .map(str::trim_start)
                .collect::<Vec<_>>()
                .join("\n");
            if data.is_empty() {
                continue;
            }
            let v: Value = serde_json::from_str(&data).map_err(|_| "invalid MCP SSE JSON")?;
            if v.get("id") == id {
                return Ok(Some((Some(v), sid)));
            }
        }
        Ok(None)
    } else {
        match serde_json::from_str::<Value>(body) {
            Ok(v) => Ok(Some((Some(v), sid))),
            Err(e) if e.is_eof() => Ok(None),
            Err(_) => Err("invalid MCP HTTP JSON".into()),
        }
    }
}
