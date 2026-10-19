use agentconductor::process::ProcessSpec;
use std::{path::PathBuf, sync::OnceLock, time::Duration};

fn cargo_tmp() -> PathBuf {
    let dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tmp");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture() -> PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            let path = cargo_tmp().join(format!("ac-mock-{}", std::process::id()));
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

pub fn spec(mode: &str) -> ProcessSpec {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cwd = cargo_tmp().join(format!("ac-run-{}-{stamp}-{sequence}", std::process::id()));
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
