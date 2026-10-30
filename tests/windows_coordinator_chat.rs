#![cfg(windows)]

use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[path = "support/chat_history_snapshot.rs"]
mod history;
#[path = "support/windows_chat_script.rs"]
mod script;

struct RunningApp(Child);
impl Drop for RunningApp {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl RunningApp {
    fn stderr_report(&mut self) -> String {
        let _ = self.0.kill();
        let _ = self.0.wait();
        let mut report = String::new();
        if let Some(mut stderr) = self.0.stderr.take() {
            let _ = stderr.read_to_string(&mut report);
        }
        report
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
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    )
}

fn drive(root: &Path, app: &mut RunningApp, question: &str, mode: &str) -> String {
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
    if !(driver.status.success() && report.contains("CHAT_VISIBLE")) {
        let child_stderr = app.stderr_report();
        panic!(
            "UI Automation exited {}; stdout: {report}; stderr: {}; child stderr: {child_stderr}; persisted state: {}",
            driver.status,
            String::from_utf8_lossy(&driver.stderr),
            diagnostic_state(root)
        );
    }
    report
}

fn diagnostic_state(root: &Path) -> String {
    let startup = fs::read_to_string(root.join("startup.json"))
        .unwrap_or_else(|error| format!("startup.json unavailable: {error}"));
    let mut jobs = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten().filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
        }) {
            let process = entry.path().join("process");
            let owner = fs::read_to_string(process.join("owner.json"))
                .unwrap_or_else(|error| format!("owner.json unavailable: {error}"));
            let stderr = fs::read_to_string(process.join("stderr.log")).unwrap_or_default();
            jobs.push(format!(
                "{} owner={owner} stderr={stderr}",
                entry.path().display()
            ));
        }
    }
    format!("startup={startup}; jobs={jobs:?}")
}

fn try_snapshot(root: &Path) -> Option<Value> {
    let bytes = fs::read(root.join("startup.json")).ok()?;
    let mut state: Value = serde_json::from_slice(&bytes).ok()?;
    state["chat_history"] = json!(history::turns(root, &state).unwrap_or_default());
    Some(state)
}

fn wait_for_transcript(root: &Path, question: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let Some(state) = try_snapshot(root) else {
            thread::sleep(Duration::from_millis(100));
            continue;
        };
        if state["chat_history"].as_array().is_some_and(|turns| {
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

fn send_without_wait(root: &Path, app: &mut RunningApp, question: &str) {
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_CHAT_MODE", "send")
        // Return after sending so the test can observe the live background chat.
        .env("AC_CHAT_WAIT", "0")
        .env("AC_CHAT_QUESTION", question)
        .env("AC_CHAT_ANSWER", "PACKAGE_CHAT_ANSWER")
        .args(["-NoProfile", "-NonInteractive", "-Command", script::SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout);
    if !(driver.status.success() && report.contains("CHAT_SENT")) {
        let child_stderr = app.stderr_report();
        panic!(
            "UI Automation send failed: stdout: {report}; stderr: {}; child stderr: {child_stderr}; persisted state: {}",
            String::from_utf8_lossy(&driver.stderr),
            diagnostic_state(root)
        );
    }
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_persists_chat_after_workspace_reopen() {
    let (_cleanup, root) = prepare_workspace("reopen", None);
    let question = "What was persisted?";
    let mut app = launch(&root);
    drive(&root, &mut app, question, "send");
    wait_for_transcript(&root, question);
    drop(app);
    let mut reopened = launch(&root);
    drive(&root, &mut reopened, question, "verify");
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_reconnects_an_in_flight_chat_after_app_close() {
    // in-flight: the worker must outlive the first UI process and finish after reopen.
    let (_cleanup, root) = prepare_workspace("resume", Some(30_000));
    let question = "Resume this answer";
    let mut app = launch(&root);
    send_without_wait(&root, &mut app, question);
    wait_for_running_chat(&root);
    drop(app);
    let mut reopened = launch(&root);
    let report = drive(&root, &mut reopened, question, "verify");
    assert!(report.contains("CHAT_RECONNECTED_IN_FLIGHT"), "{report}");
    assert!(
        report.contains("CHAT_RECONNECTED_AND_REENABLED"),
        "{report}"
    );
    wait_for_transcript(&root, question);
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_handles_a_slow_coordinator_answer_without_losing_the_chat() {
    let (_cleanup, root) = prepare_workspace("slow", Some(20_000));
    let question = "Answer after a slow response";
    let mut app = launch(&root);
    send_without_wait(&root, &mut app, question);
    wait_for_running_chat(&root);
    drive(&root, &mut app, question, "observe");
    wait_for_transcript(&root, question);
}
