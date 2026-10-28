#![cfg(windows)]

use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[path = "support/windows_desktop_script.rs"]
mod desktop_setup;
#[path = "support/windows_signal_settings_script.rs"]
mod signal_script;
#[path = "support/signal_workspace_registry.rs"]
mod workspace_registry;
struct RunningApp(Child);

impl Drop for RunningApp {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct TestDirectory(PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "requires the exact packaged Windows release binary"]
fn exact_windows_package_saves_signal_transport_and_retains_port() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ac-win-signal-{}-{nonce}", std::process::id()));
    let _cleanup = TestDirectory(root.clone());
    let home = root.join("home");
    let workspace = root.join("workspace");
    let managed_workspace = home
        .join(".agentconductor")
        .join("workspaces")
        .join("windows-uia-workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let executable = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .expect("package job must supply the exact release binary");
    let app = RunningApp(
        Command::new(executable)
            .args(["ui-new", workspace.to_str().unwrap()])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let first_workspace = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            desktop_setup::SCRIPT,
        ])
        .output()
        .unwrap();
    assert!(
        first_workspace.status.success(),
        "first workspace setup failed: stdout: {}; stderr: {}",
        String::from_utf8_lossy(&first_workspace.stdout),
        String::from_utf8_lossy(&first_workspace.stderr)
    );
    let active_workspace = workspace_registry::active_workspace(&home, &managed_workspace);
    assert!(active_workspace.join("startup.json").is_file());
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_SIGNAL_WORKSPACE", &active_workspace)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            signal_script::SCRIPT,
        ])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout);
    let persisted_settings =
        std::fs::read_to_string(active_workspace.join("signal-transport.json"))
            .unwrap_or_else(|error| format!("unavailable: {error}"));
    assert!(
        driver.status.success() && report.contains("SIGNAL_SETTINGS_SAVED"),
        "UI Automation exited {}: stdout: {report}; stderr: {}; persisted_settings={persisted_settings}",
        driver.status,
        String::from_utf8_lossy(&driver.stderr)
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let saved = loop {
        if let Ok(contents) =
            std::fs::read_to_string(active_workspace.join("signal-transport.json"))
            && let Ok(settings) = serde_json::from_str::<serde_json::Value>(&contents)
            && settings["preference"]["transport"] == "agentmemory"
            && settings["preference"]["port"] == 7333
        {
            break settings;
        }
        assert!(
            Instant::now() < deadline,
            "signal preference was not persisted"
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    assert_eq!(saved["preference"]["transport"], "agentmemory");
    assert_eq!(saved["preference"]["port"], 7333);
}
