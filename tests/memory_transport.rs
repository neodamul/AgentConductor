use agentconductor::cooperative::{Address, Progress};
use agentconductor::memory::{
    Client,
    checkpoints::{Envelope, Receipt},
};
use std::io::{Read, Write};
fn packet() -> Envelope {
    Envelope::Progress {
        progress: Progress {
            address: Address {
                run_id: "a".repeat(32),
                plan_digest: "b".repeat(64),
                task: "authentication".into(),
                agent: "company".into(),
                session_id: "company-session-1".into(),
                attempt: 1,
            },
            sequence: 1,
            reported_at_ms: 1000,
            phase: "implementation".into(),
            summary: "Working on token rotation".into(),
            next_step: "Run tests".into(),
        },
    }
}
fn request(stream: &mut std::net::TcpStream) -> (String, serde_json::Value) {
    let mut data = vec![];
    let mut buf = [0; 1024];
    loop {
        let n = stream.read(&mut buf).unwrap();
        assert!(n > 0);
        data.extend_from_slice(&buf[..n]);
        if let Some(end) = data.windows(4).position(|s| s == b"\r\n\r\n") {
            let h = String::from_utf8_lossy(&data[..end]);
            let len = h
                .lines()
                .find_map(|l| l.strip_prefix("Content-Length: "))
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap();
            if data.len() >= end + 4 + len {
                return (
                    h.into_owned(),
                    if len == 0 {
                        serde_json::Value::Null
                    } else {
                        serde_json::from_slice(&data[end + 4..end + 4 + len]).unwrap()
                    },
                );
            }
        }
    }
}
fn respond(stream: &mut std::net::TcpStream, status: u16, value: serde_json::Value) {
    let body = serde_json::to_vec(&value).unwrap();
    write!(
        stream,
        "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(&body).unwrap();
}
#[test]
fn publish_and_exact_read_validate_receipts_and_scoped_identity() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let expected = packet();
    let wire = serde_json::to_string(&expected).unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (headers, body) = request(&mut stream);
        assert!(headers.starts_with("POST /agentmemory/remember HTTP/1.1"));
        assert!(headers.contains("Authorization: Bearer fixture-secret"));
        assert_eq!(body["content"], wire);
        assert_eq!(
            body["project"],
            "agentconductor.run.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        let memory = serde_json::json!({"id":"mem_fixture_1","project":body["project"],"agentId":body["agentId"],"content":body["content"]});
        respond(
            &mut stream,
            201,
            serde_json::json!({"success":true,"memory":memory}),
        );
        drop(stream);
        let (mut stream, _) = listener.accept().unwrap();
        let (headers, _) = request(&mut stream);
        assert!(headers.starts_with("GET /agentmemory/memories/mem_fixture_1 HTTP/1.1"));
        respond(&mut stream, 200, serde_json::json!({"memory":memory}));
    });
    let client = Client::new(port, Some("fixture-secret".into())).unwrap();
    let receipt: Receipt = client.publish(&packet()).unwrap();
    assert_eq!(receipt.memory_id, "mem_fixture_1");
    assert_eq!(client.read(&receipt.memory_id).unwrap(), packet());
    server.join().unwrap();
}
#[test]
fn malformed_identity_redirects_and_oversized_responses_are_rejected() {
    assert!(Client::new(0, None).is_err());
    assert!(Client::new(1234, Some("secret\r\nInjected: header".into())).is_err());
    let client = Client::new(1234, None).unwrap();
    assert!(client.read("../private").is_err());
    for mode in [
        "redirect",
        "oversize",
        "foreign",
        "truncated",
        "conflicting_length",
    ] {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            request(&mut stream);
            match mode {
                "redirect"=>stream.write_all(b"HTTP/1.1 302 Redirect\r\nLocation: https://external.invalid/\r\nContent-Length: 0\r\n\r\n").unwrap(),
                "oversize"=>stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 99999999\r\n\r\n").unwrap(),
                "truncated"=>stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{}").unwrap(),
                "conflicting_length"=>stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Length: 20\r\n\r\n{}").unwrap(),
                _=>respond(&mut stream,200,serde_json::json!({"memory":{"id":"mem_fixture_1","project":"foreign-project","agentId":"foreign-agent","content":serde_json::to_string(&packet()).unwrap()}})),
            }
        });
        assert!(
            Client::new(port, None)
                .unwrap()
                .read("mem_fixture_1")
                .is_err(),
            "{mode}"
        );
        server.join().unwrap();
    }
}
#[test]
fn chunked_fragmented_json_is_supported_and_creation_requires_a_valid_body() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (_, body) = request(&mut stream);
        let data=serde_json::to_vec(&serde_json::json!({"success":true,"memory":{"id":"mem_fixture_2","project":body["project"],"agentId":body["agentId"],"content":body["content"]}})).unwrap();
        stream
            .write_all(b"HTTP/1.1 201 Created\r\nTransfer-Encoding: chunked\r\n\r\n")
            .unwrap();
        for chunk in data.chunks(17) {
            write!(stream, "{:x}\r\n", chunk.len()).unwrap();
            stream.write_all(chunk).unwrap();
            stream.write_all(b"\r\n").unwrap();
        }
        stream.write_all(b"0\r\n\r\n").unwrap();
        drop(stream);
        let (mut stream, _) = listener.accept().unwrap();
        request(&mut stream);
        respond(&mut stream, 201, serde_json::json!({"success":false}));
    });
    let client = Client::new(port, None).unwrap();
    assert_eq!(
        client.publish(&packet()).unwrap().memory_id,
        "mem_fixture_2"
    );
    assert!(client.publish(&packet()).is_err());
    server.join().unwrap();
}

#[test]
fn closed_connection_is_rejected_promptly_without_an_eof_spin() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        request(&mut stream);
    });
    let start = std::time::Instant::now();
    assert!(
        Client::new(port, None)
            .unwrap()
            .read("mem_fixture_1")
            .is_err()
    );
    server.join().unwrap();
    assert!(start.elapsed() < std::time::Duration::from_secs(2));
}

#[test]
fn compiled_publish_and_read_commands_match_the_typed_transport_contract() {
    let root = std::env::temp_dir().join(format!(
        "ac-memory-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("packet.json");
    std::fs::write(&path, serde_json::to_vec(&packet()).unwrap()).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (_, body) = request(&mut stream);
        let memory = serde_json::json!({"id":"mem_cli_fixture","project":body["project"],"agentId":body["agentId"],"content":body["content"]});
        respond(
            &mut stream,
            201,
            serde_json::json!({"success":true,"memory":memory}),
        );
        drop(stream);
        let (mut stream, _) = listener.accept().unwrap();
        request(&mut stream);
        respond(&mut stream, 200, serde_json::json!({"memory":memory}));
    });
    let call = |command: &str, arg: &std::ffi::OsStr| {
        std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .arg(command)
            .arg(port.to_string())
            .arg(arg)
            .env_remove("AGENTMEMORY_SECRET")
            .env("HOME", &root)
            .env("USERPROFILE", &root)
            .output()
            .unwrap()
    };
    let published = call("memory-publish", path.as_os_str());
    assert!(
        published.status.success(),
        "{}",
        String::from_utf8_lossy(&published.stderr)
    );
    let receipt: Receipt = serde_json::from_slice(&published.stdout).unwrap();
    assert_eq!(receipt.memory_id, "mem_cli_fixture");
    let read = call("memory-read", std::ffi::OsStr::new(&receipt.memory_id));
    assert!(read.status.success());
    assert_eq!(
        serde_json::from_slice::<Envelope>(&read.stdout).unwrap(),
        packet()
    );
    server.join().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
