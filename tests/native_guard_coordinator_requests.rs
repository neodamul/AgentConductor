#![cfg(unix)]
use agentconductor::{adapters, gates::Ledger, protocol::Provider, workflow};
use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;

fn start(delay: u8) -> (PathBuf, PathBuf, Child) {
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
    let plan_file = root.join("plan.json");
    fs::write(&plan_file, serde_json::to_vec(&plan).unwrap()).unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&plan).unwrap());
    let dir = root.join("run");
    let coordinator = serde_json::json!({
        "id":"slow-coordinator",
        "executable":"/bin/sh",
        "args":["-c", format!("sleep {delay}; printf '%s\\n' '{{\"type\":\"result\",\"payload\":{{\"decision\":\"allow\",\"reason\":\"routine\"}}}}'")],
        "parser":"custom",
        "input_mode":"stdin",
        "permission_mode":"read_only"
    });
    fs::write(
        agentconductor::native_guard::orchestrator_path(&dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "60")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .args([
            "run",
            plan_file.to_str().unwrap(),
            root.join("repo").to_str().unwrap(),
            dir.to_str().unwrap(),
            &digest,
        ])
        .spawn()
        .unwrap();
    (root, dir, child)
}

fn pending(dir: &std::path::Path, child: &mut Child) -> String {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(ledger) = Ledger::open(&dir.join("state.sqlite"))
            && let Some(request) = ledger
                .requests()
                .unwrap()
                .into_iter()
                .find(|request| request.status == "pending")
        {
            return request.id;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Claude tool request stayed invisible while its coordinator was deciding");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn decide(dir: &std::path::Path, action: &str, id: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args([action, dir.to_str().unwrap(), id])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn live_operator_decision_is_visible_before_coordinator_finishes_and_wins_the_race() {
    for (action, allowed) in [("tool-approve", true), ("tool-reject", false)] {
        let (root, dir, mut child) = start(15);
        let id = pending(&dir, &mut child);
        assert_eq!(workflow::load(&dir).unwrap().state, "running");
        let room = agentconductor::ui::Room::open(
            &dir,
            std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
        )
        .unwrap();
        let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        assert!(
            snapshot["tool_requests"]
                .as_array()
                .unwrap()
                .iter()
                .any(|request| { request["id"] == id && request["status"] == "pending" }),
            "task statuses: {:?}",
            snapshot["report"]["tasks"]
        );
        drop(room);
        decide(&dir, action, &id);
        assert!(child.wait().unwrap().success());
        let report = workflow::load(&dir).unwrap();
        assert_eq!(report.tasks[0].attempt, 1);
        assert_eq!(
            report.tasks[0].cwd.join("approval-marker").exists(),
            allowed
        );
        assert_eq!(report.tasks[0].cwd.join("denied-marker").exists(), !allowed);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn coordinator_decision_still_resolves_the_visible_request_when_uncontested() {
    let (root, dir, mut child) = start(1);
    assert!(child.wait().unwrap().success());
    let requests = Ledger::open(&dir.join("state.sqlite"))
        .unwrap()
        .requests()
        .unwrap();
    assert!(requests.iter().any(|request| request.status == "consumed"));
    let report = workflow::load(&dir).unwrap();
    assert!(report.tasks[0].cwd.join("approval-marker").exists());
    fs::remove_dir_all(root).unwrap();
}
