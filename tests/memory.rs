use agentconductor::{memory, state::Store};

#[test]
fn local_recall_and_outbox_survive_service_unavailability() {
    let mut store = Store::open(":memory:").unwrap();
    store
        .record(
            "workflow",
            &serde_json::json!({"state":"running"}),
            "refresh rotation workflow running",
        )
        .unwrap();
    assert_eq!(store.recall("rotation").unwrap().len(), 1);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    assert!(memory::sync(&store, port).is_err());
    assert_eq!(store.pending_memory().unwrap().len(), 1);
    assert!(store.memory_delivered(999).is_err());
}

#[test]
fn sync_marks_delivery_only_after_successful_local_http_response() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut bytes = vec![];
        let mut buf = [0; 4096];
        loop {
            let n = stream.read(&mut buf).unwrap();
            bytes.extend_from_slice(&buf[..n]);
            if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&bytes[..end]);
                let len: usize = header
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                if bytes.len() >= end + 4 + len {
                    break;
                }
            }
        }
        assert!(String::from_utf8_lossy(&bytes).starts_with("POST /agentmemory/remember HTTP/1.1"));
        let end = bytes.windows(4).position(|p| p == b"\r\n\r\n").unwrap() + 4;
        let request: serde_json::Value = serde_json::from_slice(&bytes[end..]).unwrap();
        let body=serde_json::to_vec(&serde_json::json!({"success":true,"memory":{"id":"mem_sync_fixture","project":request["project"],"content":request["content"]}})).unwrap();
        write!(
            stream,
            "HTTP/1.1 201 Created\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(&body).unwrap();
    });
    let mut store = Store::open(":memory:").unwrap();
    store
        .record("workflow", &serde_json::json!({}), "phase completed")
        .unwrap();
    assert_eq!(memory::sync(&store, port).unwrap(), 1);
    assert!(store.pending_memory().unwrap().is_empty());
    server.join().unwrap();
}

#[test]
fn a_201_status_without_a_matching_creation_receipt_keeps_the_outbox() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];
        let mut bytes = vec![];
        loop {
            let n = stream.read(&mut buf).unwrap();
            assert!(n > 0);
            bytes.extend_from_slice(&buf[..n]);
            if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&bytes[..end]);
                let len: usize = header
                    .lines()
                    .find_map(|line| line.strip_prefix("Content-Length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                if bytes.len() >= end + 4 + len {
                    break;
                }
            }
        }
        stream
            .write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .unwrap();
    });
    let mut store = Store::open(":memory:").unwrap();
    store
        .record(
            "workflow",
            &serde_json::json!({}),
            "Retain this observation",
        )
        .unwrap();
    assert!(memory::sync(&store, port).is_err());
    assert_eq!(store.pending_memory().unwrap().len(), 1);
    server.join().unwrap();
}

#[test]
fn pending_delivery_is_a_bounded_batch_and_retains_the_remainder() {
    let mut store = Store::open(":memory:").unwrap();
    for n in 0..140 {
        store
            .record(
                &format!("observation/{n}"),
                &serde_json::json!({"n":n}),
                "Progress observation",
            )
            .unwrap();
    }
    let batch = store.pending_memory().unwrap();
    assert_eq!(batch.len(), 128);
    for (id, _) in batch {
        store.memory_delivered(id).unwrap();
    }
    assert_eq!(store.pending_memory().unwrap().len(), 12);
}
