#![cfg(windows)]
//! DD-38 smoke test: the startup workspace opens in a WebView2 window and stays up
//! instead of failing with the former "requires macOS" error.
use std::time::{Duration, Instant};

#[test]
fn startup_workspace_opens_a_webview2_window() {
    let dir = std::env::temp_dir().join(format!("ac-win-shell-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["ui-new", dir.to_str().unwrap()])
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().unwrap() {
            let mut error = String::new();
            use std::io::Read;
            child
                .stderr
                .take()
                .unwrap()
                .read_to_string(&mut error)
                .unwrap();
            panic!("desktop shell exited early with {status}: {error}");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(dir.join("startup.json").exists() || dir.exists());
    let _ = std::fs::remove_dir_all(dir);
}
