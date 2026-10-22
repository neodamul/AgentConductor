#![cfg(unix)]
use agentconductor::process::{ProcessSpec, run};
use std::{path::PathBuf, process::Command, sync::OnceLock, time::Duration};
#[path = "support/liveness.rs"]
mod liveness;
#[path = "support/process_run.rs"]
mod process_run;

fn spec(mode: &str) -> ProcessSpec {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    let executable = FIXTURE
        .get_or_init(|| {
            let path = std::env::temp_dir().join(format!("ac-mock-{}", std::process::id()));
            assert!(
                Command::new("rustc")
                    .args(["tests/support/mock_agent.rs", "-o"])
                    .arg(&path)
                    .status()
                    .unwrap()
                    .success()
            );
            path
        })
        .clone();
    let cwd = std::env::temp_dir().join(format!(
        "ac-run-{}-{}-{sequence}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&cwd).unwrap();
    ProcessSpec {
        executable,
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
async fn timeout_terminates_descendants_and_reaps_direct_child() {
    let spec = spec("tree");
    let cwd = spec.cwd.clone();
    let marker = cwd.join("grandchild.pid");
    let (result, ready) = process_run::until_marker(spec, &marker, Duration::from_secs(4))
        .await
        .unwrap();
    assert!(result.timed_out);
    assert!(ready, "timeout elapsed before descendant published its PID");
    let pid: i32 = std::fs::read_to_string(&marker).unwrap().parse().unwrap();
    liveness::assert_absent(pid).await;
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn escalates_to_kill_when_interrupt_and_terminate_are_ignored() {
    let spec = spec("stubborn");
    let cwd = spec.cwd.clone();
    let (result, ready) =
        process_run::until_marker(spec, &cwd.join("ready"), Duration::from_secs(4))
            .await
            .unwrap();
    assert!(result.timed_out);
    assert_eq!(result.exit_code, None);
    assert!(
        ready,
        "timeout elapsed before the stubborn process was ready"
    );
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn timeout_is_effective_when_agent_never_reads_stdin() {
    let mut spec = spec("sleep");
    spec.input = vec![b'x'; 2 * 1024 * 1024];
    spec.timeout = Duration::from_millis(200);
    let cwd = spec.cwd.clone();
    assert!(run(spec).await.unwrap().timed_out);
    std::fs::remove_dir_all(cwd).unwrap();
}
