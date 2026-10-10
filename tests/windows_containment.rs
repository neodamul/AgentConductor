#![cfg(windows)]
//! DD-36 / QAS-04 / QAS-18: Windows process-tree containment.
use agentconductor::process::{self, Ownership, ProcessSpec};
use std::{path::PathBuf, time::Duration};

fn root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ac-win-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn spec(dir: &PathBuf, args: &[&str], input: &[u8], timeout: u64) -> ProcessSpec {
    ProcessSpec {
        executable: "powershell.exe".into(),
        args: ["-NoProfile", "-NonInteractive", "-Command"]
            .iter()
            .chain(args)
            .map(|s| s.to_string())
            .collect(),
        cwd: dir.clone(),
        input: input.to_vec(),
        log_dir: dir.join("process"),
        timeout: Duration::from_secs(timeout),
        signal_grace: Duration::from_secs(1),
        environment: Default::default(),
    }
}

fn running(pid: u32) -> bool {
    let output = std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).contains(&format!("\"{pid}\""))
}

fn owner(dir: &PathBuf) -> Ownership {
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
    let script = "$p = Start-Process -PassThru -WindowStyle Hidden ping -ArgumentList '-n','120','127.0.0.1'; \
                  Set-Content -Path grandchild.pid -Value $p.Id; Start-Sleep -Seconds 120";
    let result = process::run(spec(&dir, &[script], b"", 8)).await.unwrap();
    assert!(result.timed_out);
    let grandchild: u32 = std::fs::read_to_string(dir.join("grandchild.pid"))
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
