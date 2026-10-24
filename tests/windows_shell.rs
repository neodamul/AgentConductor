#![cfg(windows)]
//! Explicit and default startup open a native Rust desktop window sequentially.
use std::{
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

fn verify_window(mut command: Command, workspace: &Path) {
    let mut child = command
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
    assert!(workspace.is_dir(), "startup workspace was not created");
}

#[test]
fn startup_workspace_opens_a_native_window() {
    let dir = std::env::temp_dir().join(format!("ac-win-shell-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let binary = env!("CARGO_BIN_EXE_agentconductor");
    let mut explicit = Command::new(binary);
    let workspace = dir.join("explicit");
    explicit.arg("ui-new").arg(&workspace);
    verify_window(explicit, &workspace);
    for (name, empty_home) in [("missing-home", false), ("empty-home", true)] {
        let profile = dir.join(name);
        let mut default = Command::new(binary);
        default.env_remove("HOME").env("USERPROFILE", &profile);
        if empty_home {
            default.env("HOME", "");
        }
        verify_window(default, &profile.join(".agentconductor/desktop"));
    }
    std::fs::remove_dir_all(dir).unwrap();
}
