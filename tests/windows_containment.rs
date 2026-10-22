#![cfg(windows)]
//! DD-36 / QAS-04 / QAS-18: Windows process-tree containment.
use agentconductor::process::{self, Ownership, ProcessSpec};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::Duration,
};
#[path = "support/process_run.rs"]
mod process_run;

fn root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ac-win-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn spec(dir: &Path, args: &[&str], input: &[u8], timeout: u64) -> ProcessSpec {
    ProcessSpec {
        executable: "powershell.exe".into(),
        args: ["-NoProfile", "-NonInteractive", "-Command"]
            .iter()
            .chain(args)
            .map(|s| s.to_string())
            .collect(),
        cwd: dir.to_path_buf(),
        input: input.to_vec(),
        log_dir: dir.join("process"),
        timeout: Duration::from_secs(timeout),
        signal_grace: Duration::from_secs(1),
        environment: Default::default(),
    }
}

fn tree_fixture() -> PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            let mut path = std::env::temp_dir().join(format!("ac-win-tree-{}", std::process::id()));
            path.set_extension("exe");
            assert!(
                Command::new("rustc")
                    .args(["tests/support/process_tree_agent.rs", "-o"])
                    .arg(&path)
                    .status()
                    .unwrap()
                    .success()
            );
            path
        })
        .clone()
}

fn running(pid: u32) -> bool {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
}

fn owner(dir: &Path) -> Ownership {
    serde_json::from_slice(&std::fs::read(dir.join("process/owner.json")).unwrap()).unwrap()
}

#[tokio::test]
async fn contained_child_receives_stdin_and_reports_output_and_exit_code() {
    let dir = root("io");
    let result = process::run(spec(
        &dir,
        &["$text = [Console]::In.ReadToEnd(); Write-Output \"got:$text\"; exit 3"],
        b"hello",
        60,
    ))
    .await
    .unwrap();
    assert_eq!(result.exit_code, Some(3));
    assert!(!result.timed_out);
    assert_eq!(result.termination_signal, None);
    assert!(
        std::fs::read_to_string(&result.stdout)
            .unwrap()
            .contains("got:hello")
    );
    let owner = owner(&dir);
    assert_eq!(owner.phase, "terminal");
    owner.ensure_quiescent().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn timeout_terminates_the_whole_job_including_grandchildren() {
    let dir = root("tree");
    let mut spec = spec(&dir, &[], b"", 30);
    spec.executable = tree_fixture();
    spec.args = vec!["tree".into()];
    let marker = dir.join("grandchild.pid");
    let (result, ready) = process_run::until_marker(spec, &marker, Duration::from_secs(20))
        .await
        .unwrap();
    assert!(result.timed_out);
    assert!(ready, "timeout elapsed before descendant published its PID");
    let grandchild: u32 = std::fs::read_to_string(&marker)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // Termination of a job is asynchronous; allow the kernel a moment.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while running(grandchild) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(!running(grandchild), "grandchild escaped containment");
    owner(&dir).ensure_quiescent().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn stop_request_ends_a_running_child() {
    let dir = root("stop");
    let stop = dir.join("stop.request");
    let marker = stop.clone();
    let writer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        std::fs::write(marker, "stop").unwrap();
    });
    let result = process::run_until(
        spec(&dir, &["Start-Sleep -Seconds 120"], b"", 60),
        Some(stop),
    )
    .await
    .unwrap();
    writer.await.unwrap();
    assert!(result.stopped);
    assert!(!result.timed_out);
    owner(&dir).ensure_quiescent().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn npm_shell_shim_path_launches_its_cmd_sibling() {
    // npm installs `codex` (a POSIX script, os error 193 when spawned) next to `codex.cmd`.
    let dir = root("npm-shim");
    std::fs::write(dir.join("agent"), "#!/bin/sh\nexit 9\n").unwrap();
    std::fs::write(dir.join("agent.cmd"), "@echo shim:%1\r\n@exit /b 4\r\n").unwrap();
    let mut s = spec(&dir, &[], b"", 60);
    s.executable = dir.join("agent");
    s.args = vec!["ok".into()];
    let result = process::run(s).await.unwrap();
    assert_eq!(result.exit_code, Some(4));
    assert!(
        std::fs::read_to_string(&result.stdout)
            .unwrap()
            .contains("shim:ok")
    );
    std::fs::remove_dir_all(dir).unwrap();
}
