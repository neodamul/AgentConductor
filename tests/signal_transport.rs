#![cfg(unix)]
//! R-92 / DD-100 / QAS-88: Local Rust or agentmemory-preferred signal delivery with fallback.
use agentconductor::{
    cooperative::{
        self, Action, Issue,
        route::{self, Preference, Transport},
    },
    state::Store,
    workflow,
};
use std::io::{Read, Write};
use std::time::{Duration, Instant};
#[path = "support/workflow.rs"]
mod support;

fn setup() -> (std::path::PathBuf, Store, workflow::Report) {
    let (root, plan) = support::setup("shared");
    let mut report: workflow::Report = serde_json::from_value(serde_json::json!({
        "schema_version":1,"state":"running","repo":root.join("repo"),"base":"base","branch":"main", "plan":plan,
        "tasks":plan.tasks.iter().map(|t|serde_json::json!({"id":t.id,"status":"running","attempt":1,"cwd":root.join("repo"),"started_ms":100,"finished_ms":null,"session_id":format!("session-{}",t.id),"error":null,"base":"base","snapshot":null,"artifacts":root.join(&t.id)})).collect::<Vec<_>>(),
        "candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")
    })).unwrap();
    report.tasks[0].session_established = true;
    let mut store = Store::open(root.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Fixture workflow",
        )
        .unwrap();
    (root, store, report)
}
fn directive(store: &mut Store, report: &workflow::Report) -> cooperative::Directive {
    let now = 1000;
    cooperative::issue(
        store,
        Issue {
            revision: workflow::revision(report).unwrap(),
            task: report.tasks[0].id.clone(),
            action: Action::Pause,
            instruction: "Pause at the next safe checkpoint".into(),
            issued_at_ms: now,
            expires_at_ms: now + 60_000,
        },
        now,
    )
    .unwrap()
}
/// Accepts connections and answers each publish with a matching receipt; counts requests.
fn memory_server(reply: bool) -> (u16, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let seen = count.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            seen.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mut data = vec![];
            let mut buf = [0; 4096];
            let body = loop {
                let n = stream.read(&mut buf).unwrap();
                data.extend_from_slice(&buf[..n]);
                if let Some(end) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&data[..end]).to_string();
                    let len: usize = head
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length: "))
                        .unwrap()
                        .parse()
                        .unwrap();
                    if data.len() >= end + 4 + len {
                        break serde_json::from_slice::<serde_json::Value>(
                            &data[end + 4..end + 4 + len],
                        )
                        .unwrap();
                    }
                }
            };
            if !reply {
                std::thread::sleep(Duration::from_secs(12));
                continue;
            }
            let memory = serde_json::json!({"id":"mem_signal_fixture","project":body["project"],"agentId":body["agentId"],"content":body["content"]});
            let out =
                serde_json::to_vec(&serde_json::json!({"success":true,"memory":memory})).unwrap();
            write!(
                stream,
                "HTTP/1.1 201 Created\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                out.len()
            )
            .unwrap();
            stream.write_all(&out).unwrap();
        }
    });
    (port, count)
}
fn unused_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

#[test]
fn preference_defaults_to_local_and_persists() {
    let (root, _, _) = setup();
    assert_eq!(route::load_preference(&root).transport, Transport::Local);
    let chosen = Preference {
        transport: Transport::Agentmemory,
        port: 3111,
    };
    route::save_preference(&root, &chosen).unwrap();
    assert_eq!(route::load_preference(&root), chosen);
}

#[test]
fn local_selection_never_contacts_a_healthy_memory_server() {
    let (root, mut store, report) = setup();
    let (port, count) = memory_server(true);
    let d = directive(&mut store, &report);
    let r = route::deliver(
        &mut store,
        &d,
        &Preference {
            transport: Transport::Local,
            port,
        },
        &root,
    )
    .unwrap();
    assert_eq!(
        (r.preference, r.effective),
        (Transport::Local, Transport::Local)
    );
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 0);
}

#[test]
fn healthy_memory_delivers_once_and_repeats_are_idempotent() {
    let (root, mut store, report) = setup();
    let (port, count) = memory_server(true);
    let d = directive(&mut store, &report);
    let pref = Preference {
        transport: Transport::Agentmemory,
        port,
    };
    let first = route::deliver(&mut store, &d, &pref, &root).unwrap();
    assert_eq!(first.effective, Transport::Agentmemory);
    assert_eq!(first.memory_id.as_deref(), Some("mem_signal_fixture"));
    assert_eq!(route::deliver(&mut store, &d, &pref, &root).unwrap(), first);
    assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn refused_memory_falls_back_to_local_and_keeps_the_exact_directive() {
    let (root, mut store, report) = setup();
    let d = directive(&mut store, &report);
    let pref = Preference {
        transport: Transport::Agentmemory,
        port: unused_port(),
    };
    let r = route::deliver(&mut store, &d, &pref, &root).unwrap();
    assert_eq!(
        (r.preference, r.effective),
        (Transport::Agentmemory, Transport::Local)
    );
    assert!(r.cause.as_deref().unwrap().contains("agentmemory"), "{r:?}");
    // The local recipient still polls the same identity.
    let pending = cooperative::poll(&mut store, &d.address, 1200).unwrap();
    assert_eq!(pending[0].id, d.id);
    assert_eq!(route::last(&root).unwrap().effective, Transport::Local);
}

#[test]
fn silent_memory_settles_to_local_within_ten_seconds() {
    let (root, mut store, report) = setup();
    let (port, _) = memory_server(false);
    let d = directive(&mut store, &report);
    let started = Instant::now();
    let r = route::deliver(
        &mut store,
        &d,
        &Preference {
            transport: Transport::Agentmemory,
            port,
        },
        &root,
    )
    .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(r.effective, Transport::Local);
}

#[test]
fn briefing_shows_the_effective_route() {
    let (root, mut store, report) = setup();
    let d = directive(&mut store, &report);
    route::deliver(
        &mut store,
        &d,
        &Preference {
            transport: Transport::Agentmemory,
            port: unused_port(),
        },
        &root,
    )
    .unwrap();
    drop(store);
    let brief = agentconductor::briefing::collect(&root, 2000).unwrap();
    let signal = &brief["sessions"][0]["signal_route"];
    assert_eq!(signal["effective"], "local", "{brief}");
    assert_eq!(signal["preference"], "agentmemory");
}
