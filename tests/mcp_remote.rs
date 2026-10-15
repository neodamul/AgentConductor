use agentconductor::mcp::{self, Settings};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};
#[tokio::test]
async fn streamable_http_reads_sse_without_waiting_for_server_disconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let worker = std::thread::spawn(move || {
        let mut held_streams = Vec::new();
        for i in 0..4 {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = vec![];
            let mut byte = [0];
            while !bytes.ends_with(b"\r\n\r\n") {
                if stream.read(&mut byte).unwrap() == 0 {
                    break;
                }
                bytes.push(byte[0]);
            }
            let head = String::from_utf8(bytes).unwrap();
            if head.starts_with("DELETE ") {
                write!(
                    stream,
                    "HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n"
                )
                .unwrap();
                break;
            }
            let len = head
                .lines()
                .find_map(|s| {
                    s.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|s| s.trim().parse::<usize>().ok())
                })
                .unwrap();
            let mut body = vec![0; len];
            stream.read_exact(&mut body).unwrap();
            let req: Value = serde_json::from_slice(&body).unwrap();
            if i > 0 {
                assert!(
                    head.to_ascii_lowercase()
                        .contains("mcp-session-id: fixture")
                );
            }
            if req["method"] == "notifications/initialized" {
                write!(stream, "HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\n\r\n").unwrap();
                continue;
            }
            let result = if req["method"] == "initialize" {
                json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}}})
            } else {
                json!({"tools":[{"name":"get_issue","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]})
            };
            let v = json!({"jsonrpc":"2.0","id":req["id"],"result":result});
            if req["method"] == "tools/list" {
                write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nMcp-Session-Id: fixture\r\n\r\nevent: message\ndata: {v}\n\n").unwrap();
                stream.flush().unwrap();
                // Keep SSE open until DELETE: waiting for EOF must fail the request deadline.
                held_streams.push(stream);
            } else {
                let body = v.to_string();
                write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nMcp-Session-Id: fixture\r\nContent-Length: {}\r\n\r\n{}",body.len(),body).unwrap();
            }
        }
    });
    let s=Settings::parse(&json!({"mcp":{"company":{"type":"remote","url":format!("http://{addr}/mcp"),"oauth":false,"timeout":10000}}}).to_string()).unwrap();
    let result = mcp::discover(&s, "company", &std::env::current_dir().unwrap()).await;
    assert!(result.is_ok(), "{result:?}");
    worker.join().unwrap();
}
