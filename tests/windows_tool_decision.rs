#![cfg(windows)]

use agentconductor::{adapters, gates::Ledger, protocol::Provider, workflow};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[path = "support/windows_tool_decision_diagnostics.rs"]
mod diagnostics;
#[path = "support/windows_tool_decision_script.rs"]
mod script;
#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = script::SCRIPT;

struct ChildProcess(Child);

impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct TestDirectory(PathBuf);

impl std::ops::Deref for TestDirectory {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_refreshes_approves_and_rejects_tool_requests() {
    decide_with_package("refresh-approve", true);
    decide_with_package("reject", false);
}

fn decide_with_package(action: &str, allow: bool) {
    let (root, run, mut worker) = start_guarded_run();
    let initial = wait_for_request(&run, &mut worker, None);
    let home = root.join("home");
    let binary = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")));
    let app = ChildProcess(
        Command::new(binary)
            .args(["ui", run.to_str().unwrap()])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );

    let request = if action == "refresh-approve" {
        drive_ui(app.0.id(), &initial.id, "refresh");
        wait_for_status(&run, &initial.id, "superseded");
        let fresh = wait_for_request(&run, &mut worker, Some(&initial.id));
        drive_ui(app.0.id(), &fresh.id, "allow");
        fresh
    } else {
        drive_ui(app.0.id(), &initial.id, "deny");
        initial
    };

    assert!(worker.0.wait().unwrap().success(), "agent run failed");
    let ledger = Ledger::open(&run.join("state.sqlite")).unwrap();
    let receipt = ledger.receipt(&request.id).unwrap().unwrap();
    let report = workflow::load(&run).unwrap();
    let repo = root.join("repo");
    assert_eq!(receipt.status, if allow { "consumed" } else { "denied" });
    assert_eq!(repo.join("approval-marker").exists(), allow);
    assert_eq!(repo.join("denied-marker").exists(), !allow);
    assert_eq!(report.tasks[0].attempt, 1);
    assert_eq!(report.tasks[0].status, "completed");
    assert!(repo.join("one.txt").exists());
    drop(app);
}

fn start_guarded_run() -> (TestDirectory, PathBuf, ChildProcess) {
    let (root, mut plan) = support::setup("shared");
    let root = TestDirectory(root);
    let executable = root.join(if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    });
    fs::copy(support::fixture(), &executable).unwrap();
    let mut profile = plan.agents[0].clone();
    profile.id = "guarded".into();
    profile.executable = executable.to_string_lossy().into_owned();
    profile.parser = Provider::Claude;
    profile.args = adapters::invocation(Provider::Claude, None, "unused")
        .unwrap()
        .args;
    profile.permission_mode = "native_callback".into();
    plan.agents.push(profile);
    plan.tasks.truncate(1);
    plan.tasks[0].agent = "guarded".into();
    plan.tasks[0]
        .paths
        .extend(["approval-marker".into(), "denied-marker".into()]);
    plan.tasks[0].test_command = vec!["cmd.exe".into(), "/C".into(), "exit".into(), "0".into()];
    let plan_path = root.join("plan.json");
    let bytes = serde_json::to_vec(&plan).unwrap();
    fs::write(&plan_path, &bytes).unwrap();
    let digest = agentconductor::workflow::digest(&bytes);
    let run = root.join("run");
    let worker = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "60")
        .args([
            "run",
            plan_path.to_str().unwrap(),
            root.join("repo").to_str().unwrap(),
            run.to_str().unwrap(),
            &digest,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    (root, run, ChildProcess(worker))
}

fn wait_for_request(
    run: &Path,
    worker: &mut ChildProcess,
    except: Option<&str>,
) -> agentconductor::gates::Receipt {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(ledger) = Ledger::open(&run.join("state.sqlite"))
            && let Some(receipt) =
                ledger.requests().unwrap().into_iter().find(|receipt| {
                    receipt.status == "pending" && Some(receipt.id.as_str()) != except
                })
        {
            return receipt;
        }
        if let Some(status) = worker.0.try_wait().unwrap() {
            panic!("worker exited before a tool request became pending: {status}");
        }
        assert!(
            Instant::now() < deadline,
            "tool request did not become pending"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn wait_for_status(run: &Path, id: &str, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if Ledger::open(&run.join("state.sqlite"))
            .and_then(|ledger| ledger.receipt(id))
            .ok()
            .flatten()
            .is_some_and(|receipt| receipt.status == expected)
        {
            return;
        }
        if Instant::now() >= deadline {
            panic!(
                "tool receipt did not become {expected}: {}",
                refresh_failure_diagnostics(run, id)
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn refresh_failure_diagnostics(run: &Path, id: &str) -> String {
    let receipt = diagnostics::receipt_summary(
        Ledger::open(&run.join("state.sqlite")).and_then(|ledger| ledger.receipt(id)),
    );
    let report = workflow::load(run)
        .map(|report| {
            serde_json::to_string(&report)
                .unwrap_or_else(|error| format!("report serialization failed: {error}"))
        })
        .unwrap_or_else(|error| format!("report read failed: {error}"));
    let mut actions = Vec::new();
    if let Ok(entries) = fs::read_dir(run) {
        for entry in entries.flatten().filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
        }) {
            let process = entry.path().join("process");
            let owner = diagnostic_file(&process.join("owner.json"));
            let stdout = diagnostic_file(&process.join("stdout.log"));
            let stderr = diagnostic_file(&process.join("stderr.log"));
            actions.push(format!(
                "{} owner={owner} stdout={stdout} stderr={stderr}",
                entry.file_name().to_string_lossy()
            ));
        }
    }
    format!(
        "{receipt}; report={report}; actions=[{}]",
        actions.join(" | ")
    )
}

fn diagnostic_file(path: &Path) -> String {
    match fs::read(path) {
        Ok(bytes) => {
            let start = bytes.len().saturating_sub(4096);
            String::from_utf8_lossy(&bytes[start..]).into_owned()
        }
        Err(error) => format!("unavailable: {error}"),
    }
}

fn drive_ui(pid: u32, id: &str, action: &str) {
    let output = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", pid.to_string())
        .env("AC_TOOL_REQUEST_ID", id)
        .env("AC_TOOL_ACTION", action)
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("TOOL_DECISION_SUBMITTED"),
        "UI Automation failed: stdout: {stdout}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
