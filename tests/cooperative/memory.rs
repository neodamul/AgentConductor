use super::*;

fn exact_memory_server(
    packet: agentconductor::memory::checkpoints::Envelope,
    count: usize,
) -> (u16, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let a = packet.address();
    let memory = serde_json::json!({"id":"mem_import_fixture","project":format!("agentconductor.run.{}",a.run_id),"agentId":format!("agentconductor.{}",workflow::digest(&serde_json::to_vec(a).unwrap())),"content":serde_json::to_string(&packet).unwrap()});
    let server = std::thread::spawn(move || {
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = vec![];
            let mut buf = [0; 1024];
            while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut buf).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buf[..n]);
            }
            let body = serde_json::to_vec(&serde_json::json!({"memory":memory})).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        }
    });
    (port, server)
}
#[test]
fn exact_memory_ack_import_is_idempotent_and_cannot_launch_or_approve_work() {
    use agentconductor::memory::{
        Client,
        checkpoints::{self, Envelope},
    };
    let (root, mut store, report) = setup();
    let directive = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    let packet = Envelope::Acknowledgment {
        acknowledgment: ack(&directive, AckState::Accepted),
    };
    let (port, server) = exact_memory_server(packet, 2);
    let client = Client::new(port, None).unwrap();
    assert!(
        checkpoints::import(&mut store, &client, "mem_import_fixture", 1500).unwrap()["applied"]
            .as_bool()
            .unwrap()
    );
    assert!(
        !checkpoints::import(&mut store, &client, "mem_import_fixture", 1500).unwrap()["applied"]
            .as_bool()
            .unwrap()
    );
    server.join().unwrap();
    assert_eq!(
        store
            .document(&format!("cooperative_directive/{}", directive.id))
            .unwrap()["state"],
        "accepted"
    );
    assert_eq!(
        store.document("workflow").unwrap(),
        serde_json::to_value(&report).unwrap()
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn exact_memory_progress_has_real_source_and_untrusted_directives_are_not_imported() {
    use agentconductor::memory::{
        Client,
        checkpoints::{self, Envelope},
    };
    let (root, mut store, report) = setup();
    let directive = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    let progress = cooperative::Progress {
        address: directive.address.clone(),
        sequence: 1,
        reported_at_ms: 1500,
        phase: "testing".into(),
        summary: "Checking rotation failures".into(),
        next_step: "Report test evidence".into(),
    };
    let (port, server) = exact_memory_server(Envelope::Progress { progress }, 1);
    checkpoints::import(
        &mut store,
        &Client::new(port, None).unwrap(),
        "mem_import_fixture",
        1600,
    )
    .unwrap();
    server.join().unwrap();
    let bundle = agentconductor::briefing::collect(&root, 1600).unwrap();
    assert_eq!(
        bundle["sessions"][0]["memory_report"]["source"],
        "agentmemory exact-ID adapter"
    );
    let (port, server) = exact_memory_server(Envelope::Directive { directive }, 1);
    assert!(
        checkpoints::import(
            &mut store,
            &Client::new(port, None).unwrap(),
            "mem_import_fixture",
            1700
        )
        .is_err()
    );
    server.join().unwrap();
    assert_eq!(
        store.document("workflow").unwrap(),
        serde_json::to_value(&report).unwrap()
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn compiled_exact_memory_import_uses_the_same_authority_contract() {
    use agentconductor::memory::checkpoints::Envelope;
    let (root, mut store, report) = setup();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let mut request = issue(&report);
    request.issued_at_ms = now;
    request.expires_at_ms = now + 60000;
    let directive = cooperative::issue(&mut store, request, now).unwrap();
    let mut accepted = ack(&directive, AckState::Accepted);
    accepted.reported_at_ms = now;
    let (port, server) = exact_memory_server(
        Envelope::Acknowledgment {
            acknowledgment: accepted,
        },
        1,
    );
    drop(store);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("memory-import")
        .arg(&root)
        .arg(port.to_string())
        .arg("mem_import_fixture")
        .env_remove("AGENTMEMORY_SECRET")
        .env("HOME", &root)
        .env("USERPROFILE", &root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()["applied"],
        true
    );
    let store = Store::open(root.join("state.sqlite")).unwrap();
    assert_eq!(
        store.document("workflow").unwrap(),
        serde_json::to_value(&report).unwrap()
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

/// Serves the agentmemory 0.9.29 list (`GET /agentmemory/memories?agentId=…`) and exact-ID routes.
fn listing_memory_server(
    memories: Vec<serde_json::Value>,
    requests: usize,
) -> (u16, std::thread::JoinHandle<Vec<String>>) {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let mut paths = vec![];
        for _ in 0..requests {
            let (mut stream, _) = listener.accept().unwrap();
            let mut bytes = vec![];
            let mut buf = [0; 1024];
            while !bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut buf).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buf[..n]);
            }
            let line = String::from_utf8_lossy(&bytes)
                .lines()
                .next()
                .unwrap()
                .to_string();
            let path = line.split(' ').nth(1).unwrap().to_string();
            let body = if let Some(id) = path.strip_prefix("/agentmemory/memories/") {
                let memory = memories.iter().find(|m| m["id"] == id).unwrap();
                serde_json::json!({"memory":memory})
            } else {
                serde_json::json!({"memories":memories,"total":memories.len(),"offset":0,"limit":50})
            };
            let body = serde_json::to_vec(&body).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
            paths.push(path);
        }
        paths
    });
    (port, server)
}

#[test]
fn discovery_imports_addressed_observations_once_and_never_directives_or_other_runs() {
    use agentconductor::memory::{
        Client,
        checkpoints::{self, Envelope},
    };
    let (root, mut store, report) = setup();
    let directive = cooperative::issue(&mut store, issue(&report), 1000).unwrap();
    let scoped = |id: &str, packet: &Envelope| {
        let (project, agent) = checkpoints::address_scope(packet.address()).unwrap();
        serde_json::json!({"id":id,"project":project,"agentId":agent,"content":serde_json::to_string(packet).unwrap(),"isLatest":true})
    };
    let progress = Envelope::Progress {
        progress: cooperative::Progress {
            address: directive.address.clone(),
            sequence: 1,
            reported_at_ms: 1500,
            phase: "testing".into(),
            summary: "Discovered without an exact ID".into(),
            next_step: "Continue".into(),
        },
    };
    let mut foreign = scoped("mem_foreign_run", &progress);
    foreign["project"] = "agentconductor.run.ffffffffffffffffffffffffffffffff".into();
    let memories = vec![
        scoped("mem_progress_fixture", &progress),
        scoped(
            "mem_directive_fixture",
            &Envelope::Directive {
                directive: directive.clone(),
            },
        ),
        foreign,
    ];
    // First pass: list + two exact reads. Second pass: list + the directive re-read only.
    let (port, server) = listing_memory_server(memories, 5);
    let client = Client::new(port, None).unwrap();
    let first = checkpoints::discover(&mut store, &client, &directive.address, 1600).unwrap();
    assert_eq!(first["imported"], 1, "{first}");
    assert_eq!(first["skipped_directives"], 1, "{first}");
    let second = checkpoints::discover(&mut store, &client, &directive.address, 1700).unwrap();
    assert_eq!(second["imported"], 0, "{second}");
    assert_eq!(second["already_imported"], 1, "{second}");
    let paths = server.join().unwrap();
    assert!(paths[0].starts_with("/agentmemory/memories?agentId=agentconductor."));
    assert!(!paths.iter().any(|p| p.ends_with("mem_foreign_run")));
    let bundle = agentconductor::briefing::collect(&root, 1700).unwrap();
    assert_eq!(
        bundle["sessions"][0]["memory_report"]["summary"],
        "Discovered without an exact ID"
    );
    assert_eq!(
        store.document("workflow").unwrap(),
        serde_json::to_value(&report).unwrap()
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
