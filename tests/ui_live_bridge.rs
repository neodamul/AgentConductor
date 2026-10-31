#![cfg(unix)]
//! GitHub #273: the real workspace page over the real dispatcher, for checking what each step shows
//! with real jobs (planning, start, run, verification). Not part of the suite; run it on demand:
//!   cargo test --test ui_live_bridge -- --ignored --nocapture
//! then open http://127.0.0.1:8766/ in a browser. It serves for `AC_LIVE_SECONDS` (default 1200).
//! The page is the native document with its CSP removed and `window.__acRequest` posting to /rpc.
use agentconductor::ui::Room;
use serde_json::json;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    path::Path,
    process::Command,
    sync::Arc,
};
#[path = "support/workflow.rs"]
#[allow(dead_code)]
mod support;

fn git(repo: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
#[ignore = "serves the live page for a manual check"]
fn serve_the_live_page() {
    // `AC_LIVE_ROOT` picks a short folder for screenshots; it must not exist yet.
    let root = std::env::var_os("AC_LIVE_ROOT").map_or_else(
        || std::env::temp_dir().join(format!("ac-live-{}", std::process::id())),
        std::path::PathBuf::from,
    );
    let (repo, desktop, home) = (root.join("repo"), root.join("desktop"), root.join("home"));
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    // SAFETY: set before any thread of this test process reads the environment.
    unsafe { std::env::set_var("HOME", &home) };
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.name", "Test"]);
    git(&repo, &["config", "user.email", "test@example.invalid"]);
    std::fs::write(repo.join("base.txt"), "base\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "base"]);
    // A coordinator that writes a little, stays quiet for a while and then fails, like a stuck planner.
    let coordinator = root.join("coordinator.sh");
    std::fs::write(&coordinator, "#!/bin/sh\necho '{\"type\":\"system\",\"subtype\":\"init\"}'\nsleep 40\necho 'planner gave up: no plan' >&2\nexit 1\n").unwrap();
    std::fs::set_permissions(
        &coordinator,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    drop(Room::startup(&desktop, exe).unwrap());
    let path = desktop.join("startup.json");
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    saved["config"]["profiles"] = json!([
        {"id":"mock","executable":support::fixture(),"args":["task-slow"],"parser":"custom","permission_mode":"external_sandbox","max_sessions":1},
        {"id":"coordinator","executable":coordinator,"args":[],"parser":"claude","permission_mode":"read_only"}
    ]);
    saved["config"]["coordinator"] = "coordinator".into();
    saved["config"]["repository"] = json!(repo);
    saved["config"]["goal"] = "write one.txt and two.txt".into();
    saved["config"]["name"] = "live".into();
    // GitHub #277: `AC_LIVE_DEMO=1` shows a team like the manual's screenshots (Claude and Codex
    // profiles that are never run, and a local agent that does the work).
    if std::env::var_os("AC_LIVE_DEMO").is_some() {
        // Named commands: Claude and Codex never run (a plan is drafted, not generated); the local agent works.
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        for name in ["claude", "codex"] {
            std::fs::write(bin.join(name), "#!/bin/sh\nexit 0\n").unwrap();
            std::fs::set_permissions(
                bin.join(name),
                std::os::unix::fs::PermissionsExt::from_mode(0o755),
            )
            .unwrap();
        }
        std::fs::copy(support::fixture(), bin.join("local-agent")).unwrap();
        let catalog = |models: &[&str]| json!({"models":models,"source":"cli","queried_at":1793400000,"cli_version":"demo"});
        saved["config"]["profiles"] = json!([
            {"id":"claude","executable":bin.join("claude"),"args":[],"parser":"claude","permission_mode":"native_callback","max_sessions":3,"catalog":catalog(&["opus","sonnet","haiku"])},
            {"id":"codex","executable":bin.join("codex"),"args":[],"parser":"codex","permission_mode":"native_sandbox","max_sessions":3,"catalog":catalog(&["gpt-5","gpt-5-codex"])},
            {"id":"local-agent","executable":bin.join("local-agent"),"args":["task-slow"],"parser":"custom","permission_mode":"external_sandbox","max_sessions":3}
        ]);
        saved["config"]["coordinator"] = "claude".into();
        saved["config"]["goal"] =
            "Add a lightweight readiness endpoint with tests and documentation.".into();
        saved["config"]["name"] = "Status endpoint".into();
    }
    std::fs::write(&path, saved.to_string()).unwrap();
    let room = Arc::new(Room::startup(&desktop, exe).unwrap());
    // Built per request, so a language or theme change shows after a reload, as in the app.
    let page = |room: &Room| {
        room.document()
            .unwrap()
        .split("<meta http-equiv=\"Content-Security-Policy\"")
        .enumerate()
        .map(|(i, part)| if i == 0 { part.to_string() } else { part[part.find('>').unwrap() + 1..].to_string() })
        .collect::<String>()
        .replacen(
            "<head>",
            "<head><script>window.__acRequest=b=>fetch('/rpc',{method:'POST',body:b}).then(r=>r.text().then(t=>{if(!r.ok)throw Error(t);return t;}));</script>",
            1,
        )
    };
    let listener = TcpListener::bind("127.0.0.1:8766").unwrap();
    let seconds: u64 = std::env::var("AC_LIVE_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1200);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
    println!("serving http://127.0.0.1:8766/ from {}", root.display());
    listener.set_nonblocking(true).unwrap();
    while std::time::Instant::now() < deadline {
        let Ok((stream, _)) = listener.accept() else {
            std::thread::sleep(std::time::Duration::from_millis(20));
            continue;
        };
        stream.set_nonblocking(false).unwrap();
        let room = room.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let mut length = 0;
            loop {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                if header.trim().is_empty() {
                    break;
                }
                if let Some(v) = header.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = v.trim().parse().unwrap();
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            let (status, text) = if line.starts_with("POST /rpc") {
                match room.dispatch(&body) {
                    Ok(value) => ("200 OK", value.to_string()),
                    Err(error) => ("500 Error", error.to_string()),
                }
            } else {
                ("200 OK", page(&room))
            };
            let mut stream = stream;
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                text.len()
            );
        });
    }
}
