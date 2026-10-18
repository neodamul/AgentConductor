#![cfg(unix)]
use agentconductor::{adapters, gates::Ledger, protocol::Provider, workflow};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
#[path = "support/workflow.rs"]
mod support;

fn setup(mode: &str) -> (PathBuf, PathBuf) {
    let (root, mut plan) = support::setup(mode);
    let cli = root.join("claude");
    fs::copy(support::fixture(), &cli).unwrap();
    let mut profile = plan.agents[0].clone();
    profile.id = "guarded".into();
    profile.executable = cli.to_str().unwrap().into();
    profile.parser = Provider::Claude;
    profile.args = adapters::invocation(Provider::Claude, None, "unused")
        .unwrap()
        .args;
    profile.permission_mode = "native_callback".into();
    plan.agents.push(profile);
    plan.tasks[0].agent = "guarded".into();
    plan.tasks[0].paths.push("approval-marker".into());
    let path = root.join("plan.json");
    fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&plan).unwrap());
    let dir = root.join("run");
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
        .args([
            "run",
            path.to_str().unwrap(),
            root.join("repo").to_str().unwrap(),
            dir.to_str().unwrap(),
            &digest,
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    (root, dir)
}

fn command(dir: &Path, name: &str, args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
        .arg(name)
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn compiled_native_worker_stops_before_the_command_and_retains_its_session_and_exact_request() {
    for mode in ["shared", "worktree"] {
        let (root, dir) = setup(mode);
        let report = workflow::load(&dir).unwrap();
        assert_eq!(report.state, "waiting_tool_approval");
        assert_eq!(report.tasks[0].status, "waiting_approval");
        assert!(!report.tasks[0].cwd.join("approval-marker").exists());
        let requests = Ledger::open(&dir.join("state.sqlite"))
            .unwrap()
            .requests()
            .unwrap();
        let active = requests
            .iter()
            .filter(|request| matches!(request.status.as_str(), "pending" | "approved"))
            .collect::<Vec<_>>();
        assert_eq!(
            active.len(),
            1,
            "{:?}",
            requests
                .iter()
                .map(|request| &request.status)
                .collect::<Vec<_>>()
        );
        let request = active[0];
        assert_eq!(request.status, "pending");
        assert!(request.request.reason.contains("tool call command-1"));
        assert_eq!(request.request.reason.matches("tool call command-1").count(), 1);
        assert_eq!(
            report.tasks[0].tool_request.as_deref(),
            Some(request.id.as_str())
        );
        assert!(requests.iter().all(|receipt| {
            receipt.id == request.id || receipt.status == "superseded"
        }));
        if let Some(stale) = requests.iter().find(|receipt| receipt.status == "superseded") {
            let attempt = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
                .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
                .args(["tool-approve", dir.to_str().unwrap(), &stale.id])
                .output()
                .unwrap();
            assert!(!attempt.status.success());
            assert_eq!(
                Ledger::open(&dir.join("state.sqlite"))
                    .unwrap()
                    .receipt(&stale.id)
                    .unwrap()
                    .unwrap()
                    .status,
                "superseded"
            );
        }
        let refresh = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
            .args(["tool-refresh", dir.to_str().unwrap(), &request.id])
            .output()
            .unwrap();
        assert!(
            refresh.status.success(),
            "{}",
            String::from_utf8_lossy(&refresh.stderr)
        );
        let refreshed: serde_json::Value = serde_json::from_slice(&refresh.stdout).unwrap();
        let request_id = refreshed["request_sha256"].as_str().unwrap();
        assert_ne!(request_id, request.id);
        assert_eq!(request.request.tool, "Bash");
        assert_eq!(
            Some(request.context.session.as_str()),
            report.tasks[0].session_id.as_deref()
        );
        let owner: agentconductor::process::Ownership = serde_json::from_slice(
            &fs::read(report.tasks[0].artifacts.join("process/owner.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(owner.phase, "terminal");
        owner.ensure_quiescent().unwrap();
        assert!(workflow::approve(&dir, "invented").is_err());
        let approve = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
            .args(["tool-approve", dir.to_str().unwrap(), request_id])
            .output()
            .unwrap();
        assert!(
            approve.status.success(),
            "{}",
            String::from_utf8_lossy(&approve.stderr)
        );
        let resume = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
            .args(["resume-due", dir.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            resume.status.success(),
            "{}",
            String::from_utf8_lossy(&resume.stderr)
        );
        let resumed = workflow::load(&dir).unwrap();
        assert_eq!(resumed.state, "waiting_approval");
        assert_eq!(resumed.tasks[0].session_id, report.tasks[0].session_id);
        assert!(resumed.tasks[0].cwd.join("approval-marker").exists());
        assert!(
            Ledger::open(&dir.join("state.sqlite"))
                .unwrap()
                .requests()
                .unwrap()
                .iter()
                .any(|r| r.id == request_id && r.status == "consumed")
        );
        assert!(workflow::merge(&dir).is_err());
        workflow::approve(&dir, &resumed.candidate.unwrap().commit).unwrap();
        workflow::merge(&dir).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn rejection_and_stop_require_explicit_human_intervention_before_a_native_retry() {
    let (root, dir) = setup("shared");
    let id = workflow::load(&dir).unwrap().tasks[0]
        .tool_request
        .clone()
        .unwrap();
    command(&dir, "tool-reject", &[&id]);
    command(&dir, "resume-due", &[]);
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "blocked");
    assert_eq!(report.tasks[0].attempt, 1);
    assert!(!report.tasks[0].cwd.join("approval-marker").exists());
    command(&dir, "revise", &["one.txt", "Skip the command"]);
    command(&dir, "resume-due", &[]);
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(!report.tasks[0].cwd.join("approval-marker").exists());
    fs::remove_dir_all(root).unwrap();

    let (root, dir) = setup("shared");
    let id = workflow::load(&dir).unwrap().tasks[0]
        .tool_request
        .clone()
        .unwrap();
    command(&dir, "stop", &["Operator review"]);
    command(&dir, "tool-approve", &[&id]);
    assert_eq!(workflow::load(&dir).unwrap().state, "stopped");
    command(&dir, "resume-due", &[]);
    assert_eq!(workflow::load(&dir).unwrap().tasks[0].attempt, 1);
    command(&dir, "resume", &[]);
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_expired_approval_stays_unusable_until_a_fresh_pending_request_is_reviewed() {
    let (root, dir) = setup("shared");
    let first = workflow::load(&dir).unwrap().tasks[0]
        .tool_request
        .clone()
        .unwrap();
    command(&dir, "tool-approve", &[&first, "1"]);
    std::thread::sleep(std::time::Duration::from_millis(1100));
    command(&dir, "resume-due", &[]);
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "waiting_tool_approval");
    assert!(!report.tasks[0].cwd.join("approval-marker").exists());
    let refresh = command(&dir, "tool-refresh", &[&first]);
    let response: serde_json::Value = serde_json::from_slice(&refresh.stdout).unwrap();
    let fresh = response["request_sha256"].as_str().unwrap();
    assert_ne!(fresh, first);
    command(&dir, "resume-due", &[]);
    assert!(
        !workflow::load(&dir).unwrap().tasks[0]
            .cwd
            .join("approval-marker")
            .exists()
    );
    command(&dir, "tool-approve", &[fresh]);
    command(&dir, "resume-due", &[]);
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn changing_the_paused_workspace_refuses_the_recorded_human_decision() {
    let (root, dir) = setup("shared");
    let report = workflow::load(&dir).unwrap();
    let id = report.tasks[0].tool_request.as_deref().unwrap();
    fs::write(report.repo.join("one.txt"), "External edit").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "0")
        .arg("tool-approve")
        .arg(&dir)
        .arg(id)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        Ledger::open(&dir.join("state.sqlite"))
            .unwrap()
            .receipt(id)
            .unwrap()
            .unwrap()
            .status,
        "pending"
    );
    assert!(!report.tasks[0].cwd.join("approval-marker").exists());
    fs::remove_dir_all(root).unwrap();
}
