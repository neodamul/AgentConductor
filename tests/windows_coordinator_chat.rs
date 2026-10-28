#![cfg(windows)]

use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[path = "support/windows_chat_script.rs"]
mod script;

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
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture(root: &Path) -> PathBuf {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/chat_agent.rs");
    let path = root.join("chat-agent.exe");
    assert!(
        Command::new("rustc")
            .args(["--edition", "2024", source.to_str().unwrap(), "-o"])
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    path
}

fn configure(root: &Path, project: &Path, executable: &Path, delay_ms: Option<u64>) {
    let startup = Startup::open(root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut profile = json!({
        "id":"chat-mock", "parser":"custom", "executable":executable,
        "args":[], "input_mode":"stdin", "permission_mode":"read_only",
        "event_types":{"result":"result"}, "max_sessions":1,
        "quota_refresh_minutes":5
    });
    if let Some(delay) = delay_ms {
        profile["environment"] = json!({"AC_CHAT_FIXTURE_DELAY_MS": delay.to_string()});
    }
    let mut config = state["config"].clone();
    config["profiles"] = json!([profile]);
    config["coordinator"] = "chat-mock".into();
    config["repository"] = project.to_string_lossy().into_owned().into();
    config["name"] = "Package chat".into();
    config["goal"] = "Verify coordinator chat persistence".into();
    startup
        .dispatch(
            &json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .into_bytes(),
        )
        .unwrap();
}

fn launch(root: &Path) -> RunningApp {
    let binary = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .expect("package job must supply AGENTCONDUCTOR_DESKTOP_BINARY");
    RunningApp(
        Command::new(binary)
            .args(["ui-new", root.to_str().unwrap()])
            .env("HOME", root.join("home"))
            .env("USERPROFILE", root.join("home"))
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    )
}

fn drive(app: &RunningApp, question: &str, mode: &str) -> String {
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_CHAT_MODE", mode)
        .env("AC_CHAT_WAIT", "1")
        .env("AC_CHAT_QUESTION", question)
        .env("AC_CHAT_ANSWER", "PACKAGE_CHAT_ANSWER")
        .args(["-NoProfile", "-NonInteractive", "-Command", script::SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout).into_owned();
    assert!(
        driver.status.success() && report.contains("CHAT_VISIBLE"),
        "UI Automation exited {}; stdout: {report}; stderr: {}",
        driver.status,
        String::from_utf8_lossy(&driver.stderr)
    );
    report
}

fn try_snapshot(root: &Path) -> Option<Value> {
    fs::read(root.join("startup.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

fn wait_for_transcript(root: &Path, question: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let Some(state) = try_snapshot(root) else {
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        if state["chat_transcript"].as_array().is_some_and(|turns| {
            turns.iter().any(|turn| turn["text"] == question)
                && turns
                    .iter()
                    .any(|turn| turn["text"] == "PACKAGE_CHAT_ANSWER")
        }) {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!(
        "chat transcript was not persisted: {:?}",
        try_snapshot(root)
    );
}

fn wait_for_running_chat(root: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let Some(state) = try_snapshot(root) else {
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        if state["job"]["operation"] == "chat"
            && fs::read_dir(root)
                .unwrap()
                .flatten()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("coordinator-ui-")
                })
                .any(|entry| {
                    fs::read_to_string(entry.path().join("process/owner.json"))
                        .ok()
                        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                        .is_some_and(|owner| owner["phase"] == "running")
                })
        {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("chat job did not become running: {:?}", try_snapshot(root));
}

fn prepare_workspace(name: &str, delay_ms: Option<u64>) -> (TestDirectory, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "ac-win-chat-{name}-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let project = root.join("project");
    fs::create_dir_all(&project).unwrap();
    let fixture = fixture(&root);
    configure(&root, &project, &fixture, delay_ms);
    (TestDirectory(root.clone()), root)
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_persists_chat_after_workspace_reopen() {
    let (_cleanup, root) = prepare_workspace("reopen", None);
    let question = "What was persisted?";
    let app = launch(&root);
    drive(&app, question, "send");
    wait_for_transcript(&root, question);
    drop(app);
    let reopened = launch(&root);
    drive(&reopened, question, "verify");
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_reconnects_an_in_flight_chat_after_app_close() {
    // in-flight: the worker must outlive the first UI process and finish after reopen.
    let (_cleanup, root) = prepare_workspace("resume", Some(5000));
    let question = "Resume this answer";
    let app = launch(&root);
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_CHAT_MODE", "send")
        .env("AC_CHAT_WAIT", "0")
        .env("AC_CHAT_QUESTION", question)
        .env("AC_CHAT_ANSWER", "PACKAGE_CHAT_ANSWER")
        .args(["-NoProfile", "-NonInteractive", "-Command", script::SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout);
    assert!(
        driver.status.success() && report.contains("CHAT_SENT"),
        "UI Automation send failed: stdout: {report}; stderr: {}",
        String::from_utf8_lossy(&driver.stderr)
    );
    wait_for_running_chat(&root);
    drop(app);
    let reopened = launch(&root);
    drive(&reopened, question, "verify");
    wait_for_transcript(&root, question);
}
