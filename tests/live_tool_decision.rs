#![cfg(unix)]
use agentconductor::{adapters, gates::Ledger, protocol::Provider, workflow};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;

fn start() -> (PathBuf, PathBuf, Child) {
    start_with(&[])
}

fn start_with(env: &[(&str, &str)]) -> (PathBuf, PathBuf, Child) {
    let (root, mut plan) = support::setup("shared");
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
    plan.tasks[0].paths.push("denied-marker".into());
    let path = root.join("plan.json");
    fs::write(&path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&plan).unwrap());
    let dir = root.join("run");
    let child = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "60")
        .envs(env.iter().copied())
        .args([
            "run",
            path.to_str().unwrap(),
            root.join("repo").to_str().unwrap(),
            dir.to_str().unwrap(),
            &digest,
        ])
        .spawn()
        .unwrap();
    (root, dir, child)
}

fn pending(dir: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if dir.join("state.sqlite").exists()
            && let Ok(ledger) = Ledger::open(&dir.join("state.sqlite"))
            && let Some(r) = ledger
                .requests()
                .unwrap()
                .into_iter()
                .find(|r| r.status == "pending")
        {
            return r.id;
        }
        assert!(Instant::now() < deadline, "no live pending tool request");
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn decide(dir: &Path, command: &str, id: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args([command, dir.to_str().unwrap(), id])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn live_approval_lets_the_waiting_worker_run_the_exact_command_in_the_same_attempt() {
    let (root, dir, mut child) = start();
    let id = pending(&dir);
    assert_eq!(workflow::load(&dir).unwrap().state, "running");
    decide(&dir, "tool-approve", &id);
    assert!(child.wait().unwrap().success());
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.tasks[0].attempt, 1);
    assert!(report.tasks[0].cwd.join("approval-marker").exists());
    assert!(
        Ledger::open(&dir.join("state.sqlite"))
            .unwrap()
            .requests()
            .unwrap()
            .iter()
            .any(|r| r.status == "consumed")
    );
    assert!(workflow::merge(&dir).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn live_rejection_denies_only_that_call_and_the_worker_continues() {
    let (root, dir, mut child) = start();
    let id = pending(&dir);
    decide(&dir, "tool-reject", &id);
    assert!(child.wait().unwrap().success());
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert_eq!(report.tasks[0].attempt, 1);
    assert!(!report.tasks[0].cwd.join("approval-marker").exists());
    assert!(report.tasks[0].cwd.join("denied-marker").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_agent_room_lists_a_request_its_running_worker_is_paused_on() {
    let (root, dir, mut child) = start();
    let id = pending(&dir);
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::open(&dir, exe).unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let listed = snapshot["tool_requests"].as_array().unwrap();
    assert!(
        listed
            .iter()
            .any(|r| r["id"] == id.as_str() && r["status"] == "pending"),
        "live request missing from the Room: {listed:?}"
    );
    drop(room);
    decide(&dir, "tool-reject", &id);
    assert!(child.wait().unwrap().success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_identical_command_in_a_later_tool_call_needs_its_own_decision() {
    let (root, dir, mut child) = start_with(&[("MOCK_REPEAT_COMMAND", "1")]);
    let first = pending(&dir);
    // Let the other shared-workspace writers finish so both calls see the same state.
    let settle = Instant::now() + Duration::from_secs(20);
    while !["two.txt", "three.txt"]
        .iter()
        .all(|f| root.join("repo").join(f).exists())
    {
        assert!(Instant::now() < settle);
        std::thread::sleep(Duration::from_millis(100));
    }
    std::thread::sleep(Duration::from_millis(500));
    decide(&dir, "tool-approve", &first);
    let deadline = Instant::now() + Duration::from_secs(20);
    let second = loop {
        if let Some(r) = Ledger::open(&dir.join("state.sqlite"))
            .unwrap()
            .requests()
            .unwrap()
            .into_iter()
            .find(|r| r.status == "pending" && r.id != first)
        {
            break r.id;
        }
        assert!(
            Instant::now() < deadline,
            "second call never became decidable"
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    decide(&dir, "tool-approve", &second);
    assert!(child.wait().unwrap().success());
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.tasks[0].attempt, 1);
    assert_eq!(
        fs::read_to_string(report.tasks[0].cwd.join("approval-marker")).unwrap(),
        "approved command ran 2"
    );
    fs::remove_dir_all(root).unwrap();
}
