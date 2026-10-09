#![cfg(unix)]
use agentconductor::process::{ProcessSpec, run};
use std::{path::PathBuf, sync::OnceLock, time::Duration};

fn fixture() -> PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            let path = std::env::temp_dir().join(format!("ac-mock-{}", std::process::id()));
            let status = std::process::Command::new("rustc")
                .args(["tests/support/mock_agent.rs", "-o"])
                .arg(&path)
                .status()
                .unwrap();
            assert!(status.success());
            path
        })
        .clone()
}

fn spec(mode: &str) -> ProcessSpec {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cwd =
        std::env::temp_dir().join(format!("ac-run-{}-{stamp}-{sequence}", std::process::id()));
    std::fs::create_dir(&cwd).unwrap();
    ProcessSpec {
        executable: fixture(),
        args: vec![mode.into()],
        cwd: cwd.clone(),
        input: "{\"text\":\"hello 🦀\"}\n".as_bytes().to_vec(),
        log_dir: cwd.join("logs"),
        timeout: Duration::from_secs(5),
        signal_grace: Duration::from_millis(50),
        environment: Default::default(),
    }
}

#[tokio::test]
async fn closes_stdin_and_drains_large_stderr_preserving_raw_bytes() {
    let mut s = spec("echo");
    s.input = s.input.repeat(100_000);
    let cwd = s.cwd.clone();
    let input = s.input.clone();
    let result = run(s).await.unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.timed_out);
    assert_eq!(std::fs::read(result.stdout).unwrap(), input);
    assert_eq!(
        std::fs::metadata(result.stderr).unwrap().len(),
        2 * 1024 * 1024
    );
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn records_nonzero_exit_without_retrying() {
    let s = spec("fail");
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert_eq!(result.exit_code, Some(7));
    assert!(!result.timed_out);
    let owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
    assert_eq!(owner["phase"], "terminal");
    assert_eq!(owner["exit_code"], 7);
    assert!(owner["pid"].as_u64().unwrap() > 0);
    assert_eq!(owner["pid"], owner["process_group"]);
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn timeout_terminates_descendants_and_reaps_direct_child() {
    let mut s = spec("tree");
    s.timeout = Duration::from_millis(500);
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert!(result.timed_out);
    let pid: i32 = std::fs::read_to_string(cwd.join("grandchild.pid"))
        .unwrap()
        .parse()
        .unwrap();
    // A zombie is terminated but may await the host's orphan reaper.
    let output = std::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    let state = String::from_utf8_lossy(&output.stdout);
    assert!(
        state.trim().is_empty() || state.trim().starts_with('Z'),
        "surviving descendant: {state}"
    );
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn reports_spawn_failure() {
    let mut s = spec("echo");
    let cwd = s.cwd.clone();
    s.executable = cwd.join("missing");
    assert!(run(s).await.is_err());
    let owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
    assert_eq!(owner["phase"], "not_spawned");
    assert!(owner["pid"].is_null());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn timeout_is_effective_when_agent_never_reads_stdin() {
    let mut s = spec("sleep");
    s.input = vec![b'x'; 2 * 1024 * 1024];
    s.timeout = Duration::from_millis(200);
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert!(result.timed_out);
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn escalates_to_kill_when_interrupt_and_terminate_are_ignored() {
    let mut s = spec("stubborn");
    s.timeout = Duration::from_millis(500);
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert!(result.timed_out);
    assert_eq!(result.exit_code, None);
    assert!(cwd.join("ready").exists());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn explicit_stop_terminates_process_tree_and_is_distinct_from_timeout() {
    let s = spec("tree");
    let cwd = s.cwd.clone();
    let marker = cwd.join("stop.request");
    let request = marker.clone();
    let ready = cwd.join("grandchild.pid");
    let sender = tokio::spawn(async move {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        std::fs::write(request, "human stop").unwrap();
    });
    let result = agentconductor::process::run_until(s, Some(marker))
        .await
        .unwrap();
    sender.await.unwrap();
    assert!(result.stopped);
    assert!(!result.timed_out);
    let pid = std::fs::read_to_string(cwd.join("grandchild.pid")).unwrap();
    let output = std::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid])
        .output()
        .unwrap();
    let state = String::from_utf8_lossy(&output.stdout);
    assert!(state.trim().is_empty() || state.trim().starts_with('Z'));
    assert!(result.stdout.exists() && result.stderr.exists());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn stop_requested_before_launch_does_not_spawn_agent() {
    let s = spec("stubborn");
    let cwd = s.cwd.clone();
    let marker = cwd.join("stop.request");
    std::fs::write(&marker, "stop").unwrap();
    let result = agentconductor::process::run_until(s, Some(marker))
        .await
        .unwrap();
    assert!(result.stopped);
    assert!(!cwd.join("ready").exists());
    let owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
    assert_eq!(owner["phase"], "not_spawned");
    std::fs::remove_dir_all(cwd).unwrap();
}
