#![cfg(unix)]
use agentconductor::{ui::Server, workflow};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
#[path = "support/workflow.rs"]
mod support;

fn request(
    server: &Server,
    path: &str,
    cookie: bool,
    origin: Option<&str>,
    body: Option<&str>,
) -> String {
    let address = server.address();
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let method = if body.is_some() { "POST" } else { "GET" };
    let body = body.unwrap_or("");
    let cookie = if cookie {
        format!(
            "Cookie: ac_session_{}={}\r\n",
            address.port(),
            server.capability()
        )
    } else {
        String::new()
    };
    let origin = origin
        .map(|s| format!("Origin: {s}\r\n"))
        .unwrap_or_default();
    write!(stream,"{method} {path} HTTP/1.1\r\nHost: {address}\r\n{cookie}{origin}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}
fn json(response: &str) -> serde_json::Value {
    serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
}

#[tokio::test]
async fn local_ui_requires_capability_origin_and_current_revision_before_mutating() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let server = Server::start(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    assert!(request(&server, "/api/snapshot", false, None, None).starts_with("HTTP/1.1 403"));
    let state = json(&request(&server, "/api/snapshot", true, None, None));
    assert_eq!(state["report"]["state"], "waiting_approval");
    let candidate = state["report"]["candidate"]["commit"].as_str().unwrap();
    let revision = state["revision"].as_str().unwrap();
    let action =
        serde_json::json!({"revision":revision,"action":{"kind":"approve","sha":candidate}})
            .to_string();
    assert!(
        request(
            &server,
            "/api/action",
            true,
            Some("https://foreign.invalid"),
            Some(&action)
        )
        .starts_with("HTTP/1.1 403")
    );
    assert!(request(&server, "/api/action", true, None, Some(&action)).starts_with("HTTP/1.1 403"));
    let origin = format!("http://{}", server.address());
    let stale =
        serde_json::json!({"revision":"0".repeat(64),"action":{"kind":"approve","sha":candidate}})
            .to_string();
    assert!(
        request(&server, "/api/action", true, Some(&origin), Some(&stale))
            .starts_with("HTTP/1.1 409")
    );
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    assert!(
        request(&server, "/api/action", true, Some(&origin), Some(&action))
            .starts_with("HTTP/1.1 202")
    );
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let state = json(&request(&server, "/api/snapshot", true, None, None));
        if state["job"]["state"] == "completed" {
            assert_eq!(state["report"]["state"], "approved");
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(workflow::load(&dir).unwrap().state == "approved");
    let evidence = json(&request(&server, "/api/evidence", true, None, None));
    assert!(evidence["diff"].as_str().unwrap().contains("one.txt"));
    assert_eq!(evidence["report"]["tests"].as_array().unwrap().len(), 3);
    assert!(request(&server, "/ui.js", true, None, None).contains("textContent"));
    let current = json(&request(&server, "/api/snapshot", true, None, None));
    let bad = serde_json::json!({"revision":current["revision"],"action":{"kind":"merge","sha":"wrong-candidate"}}).to_string();
    assert!(
        request(&server, "/api/action", true, Some(&origin), Some(&bad))
            .starts_with("HTTP/1.1 409")
    );
    let merge = serde_json::json!({"revision":current["revision"],"action":{"kind":"merge","sha":candidate}}).to_string();
    let accepted = request(&server, "/api/action", true, Some(&origin), Some(&merge));
    assert!(accepted.starts_with("HTTP/1.1 202"));
    let id = json(&accepted)["id"].as_str().unwrap().to_owned();
    let owner: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(format!("coordinator-ui-{id}/process/owner.json"))).unwrap(),
    )
    .unwrap();
    let pid = owner["pid"].as_i64().unwrap() as libc::pid_t;
    // Closing the surface must not cancel the accepted integration process.
    drop(server);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while workflow::load(&dir).unwrap().state != "completed" {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut status = 0;
    assert_eq!(unsafe { libc::waitpid(pid, &mut status, 0) }, pid);
    assert_eq!(libc::WEXITSTATUS(status), 0);
    let git = agentconductor::git::Repository::open(&root.join("repo")).unwrap();
    assert_eq!(git.head().unwrap(), candidate);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn broker_revision_is_rechecked_by_the_child_under_the_workflow_lock() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let revision = workflow::revision(&report).unwrap();
    workflow::revise(&dir, "one.txt", "Additional edge case").unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args([
            "ui-action",
            dir.to_str().unwrap(),
            &revision,
            "approve",
            &report.candidate.unwrap().commit,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("changed since review"));
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn independent_ui_servers_use_distinct_browser_cookie_names() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let executable = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let first = Server::start(&dir, executable).unwrap();
    let second = Server::start(&dir, executable).unwrap();
    let cookie = |server: &Server| {
        let response = request(
            server,
            &format!("/session/{}", server.capability()),
            false,
            None,
            None,
        );
        assert!(response.starts_with("HTTP/1.1 303"));
        response
            .lines()
            .find_map(|line| line.strip_prefix("Set-Cookie: "))
            .unwrap()
            .split('=')
            .next()
            .unwrap()
            .to_owned()
    };
    assert_ne!(
        cookie(&first),
        cookie(&second),
        "Cookies are scoped to host, not port"
    );
    drop(first);
    drop(second);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn local_transport_rejects_ambiguous_and_unbounded_requests() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let server = Server::start(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    for (headers, status) in [
        ("Host: other.invalid\r\n", 400),
        ("Transfer-Encoding: chunked\r\n", 400),
        ("Content-Length: 65537\r\n", 400),
        ("Sec-Fetch-Site: cross-site\r\n", 403),
    ] {
        let mut stream = TcpStream::connect(server.address()).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(
            stream,
            "GET /api/snapshot HTTP/1.1\r\nHost: {}\r\nCookie: ac_session_{}={}\r\n{headers}\r\n",
            server.address(),
            server.address().port(),
            server.capability()
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(
            response.starts_with(&format!("HTTP/1.1 {status}")),
            "{response}"
        );
    }
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    drop(server);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn ui_evidence_does_not_publish_profile_environment_or_follow_artifact_aliases() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0]
        .environment
        .insert("INTERNAL_CREDENTIAL".into(), "fixture-private-value".into());
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let server = Server::start(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let evidence = request(&server, "/api/evidence", true, None, None);
    assert!(!evidence.contains("fixture-private-value"));
    let stdout = workflow::load(&dir).unwrap().tests[0].stdout.clone();
    let outside = root.join("private.log");
    fs::write(&outside, "fixture-private-log").unwrap();
    fs::rename(&stdout, stdout.with_extension("retained")).unwrap();
    std::os::unix::fs::symlink(&outside, &stdout).unwrap();
    let evidence = request(&server, "/api/evidence", true, None, None);
    assert!(evidence.starts_with("HTTP/1.1 409"));
    assert!(!evidence.contains("fixture-private-log"));
    drop(server);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_run_can_have_only_one_native_room_and_release_allows_reopening() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let first = Server::start_window(&dir, exe).unwrap();
    assert!(Server::start_window(&dir, exe).is_err());
    let child = std::process::Command::new(exe)
        .args(["ui", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!child.status.success());
    assert!(String::from_utf8_lossy(&child.stderr).contains("Agent Room is already open"));
    drop(first);
    let reopened = Server::start_window(&dir, exe).unwrap();
    drop(reopened);
    fs::remove_dir_all(root).unwrap();
}
