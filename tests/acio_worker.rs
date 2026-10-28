#![cfg(unix)]

#[path = "support/workflow.rs"]
mod support;

use agentconductor::{gates::Ledger, protocol::Protocol, workflow};
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command},
    thread,
    time::{Duration, Instant},
};

fn script(root: &Path, name: &str, body: &str) -> PathBuf {
    let executable = root.join(name);
    std::fs::write(&executable, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    executable
}

fn plan(mut plan: agentconductor::plan::Plan, executable: &Path) -> agentconductor::plan::Plan {
    let mut agent = plan.agents[0].clone();
    agent.executable = executable.to_string_lossy().into_owned();
    agent.protocol = Some(Protocol::Acio);
    plan.agents = vec![agent];
    plan.tasks.truncate(1);
    plan.tasks[0].paths = vec!["one.txt".into()];
    plan.tasks[0].test_command = vec!["/bin/sh".into(), "-c".into(), "test -s one.txt".into()];
    plan
}

const EVENT_HEADER: &str = r#"n=0
emit() { n=$((n+1)); printf '{"schema":"acio.event/1","seq":%s,"ts":"2026-10-29T00:00:00Z","session_id":"%s","type":"%s","data":%s}\n' "$n" "$session_id" "$1" "$2"; }
IFS= read -r input
session_id=$(printf '%s' "$input" | sed -n 's/.*"session":{"id":"\([^"]*\)".*/\1/p')
"#;

#[test]
fn opted_in_worker_sends_structured_input_and_persists_session_and_events() {
    let (root, initial) = support::setup("shared");
    let capture = root.join("captured-input.json");
    let executable = script(
        &root,
        "acio-worker.sh",
        &format!(
            "{EVENT_HEADER}printf '%s' \"$input\" > \"$ACIO_CAPTURE\"\ncase \"$input\" in *'\"schema\":\"acio.input/1\"'*) ;; *) exit 9;; esac\nemit session \"{{\\\"session_id\\\":\\\"$session_id\\\"}}\"\nprintf 'implemented\\n' > one.txt\nemit progress '{{\"message\":\"wrote one.txt\"}}'\nemit result '{{\"status\":\"success\",\"summary\":\"finished\",\"changed_paths\":[\"one.txt\"]}}'\n"
        ),
    );
    let mut plan = plan(initial, &executable);
    plan.agents[0].environment.insert(
        "ACIO_CAPTURE".into(),
        capture.to_string_lossy().into_owned(),
    );

    let report = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(workflow::run(
            plan,
            &root.join("repo"),
            &root.join("run"),
            20,
        ))
        .unwrap();

    assert_eq!(
        report.tasks[0].status, "completed",
        "{:?}",
        report.tasks[0].error
    );
    let input: serde_json::Value =
        serde_json::from_slice(&std::fs::read(capture).unwrap()).unwrap();
    agentconductor::cli_io::validate("input", &input).unwrap();
    assert!(
        input["instruction"]["text"]
            .as_str()
            .unwrap()
            .contains("Task: one.txt")
    );
    assert_eq!(input["workspace"]["paths"][0], "one.txt");
    assert_eq!(
        report.tasks[0].session_id.as_deref(),
        input["session"]["id"].as_str()
    );
    assert!(report.tasks[0].session_established);
    let events = std::fs::read_to_string(root.join("run/task-0/events.jsonl")).unwrap();
    assert!(events.contains(input["session"]["id"].as_str().unwrap()));
    assert!(events.contains("finished"));
    std::fs::remove_dir_all(root).unwrap();
}

fn launch(
    root: &Path,
    initial: agentconductor::plan::Plan,
    executable: &Path,
    decision_wait: &str,
) -> (PathBuf, Child) {
    let plan = plan(initial, executable);
    let plan_file = root.join("plan.json");
    std::fs::write(&plan_file, serde_json::to_vec(&plan).unwrap()).unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&plan).unwrap());
    let run = root.join("run");
    let child = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", decision_wait)
        .args([
            "run",
            plan_file.to_str().unwrap(),
            root.join("repo").to_str().unwrap(),
            run.to_str().unwrap(),
            &digest,
        ])
        .spawn()
        .unwrap();
    (run, child)
}

#[test]
fn requested_acio_tool_waits_for_an_exact_human_approval() {
    let (root, initial) = support::setup("shared");
    let executable = script(
        &root,
        "acio-approval-worker.sh",
        &format!(
            "{EVENT_HEADER}emit session \"{{\\\"session_id\\\":\\\"$session_id\\\"}}\"\nemit tool '{{\"call_id\":\"call-1\",\"name\":\"file.write\",\"status\":\"requested\",\"needs_decision\":true,\"input\":{{\"path\":\"one.txt\"}}}}'\nIFS= read -r control\ncase \"$control\" in *'\"decision\":\"allow\"'*) printf 'approved\\n' > one.txt; emit tool '{{\"call_id\":\"call-1\",\"name\":\"file.write\",\"status\":\"completed\"}}'; emit result '{{\"status\":\"success\",\"summary\":\"approved\"}}';; *) emit tool '{{\"call_id\":\"call-1\",\"name\":\"file.write\",\"status\":\"denied\"}}'; emit result '{{\"status\":\"failure\",\"summary\":\"denied\"}}';; esac\n"
        ),
    );
    let (run, mut child) = launch(&root, initial, &executable, "30");
    let deadline = Instant::now() + Duration::from_secs(10);
    let id = loop {
        if let Ok(ledger) = Ledger::open(&run.join("state.sqlite"))
            && let Some(request) = ledger
                .requests()
                .unwrap()
                .into_iter()
                .find(|request| request.status == "pending")
        {
            break request.id;
        }
        assert!(
            child.try_wait().unwrap().is_none(),
            "worker exited before creating a human approval request"
        );
        assert!(
            Instant::now() < deadline,
            "no ACIO approval request appeared"
        );
        thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(workflow::load(&run).unwrap().state, "running");
    assert_eq!(workflow::load(&run).unwrap().tasks[0].status, "running");
    let approved = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["tool-approve", run.to_str().unwrap(), &id])
        .output()
        .unwrap();
    assert!(
        approved.status.success(),
        "{}",
        String::from_utf8_lossy(&approved.stderr)
    );
    assert!(child.wait().unwrap().success());
    assert!(root.join("repo/one.txt").exists());
    assert!(
        Ledger::open(&run.join("state.sqlite"))
            .unwrap()
            .requests()
            .unwrap()
            .iter()
            .any(|request| request.id == id && request.status == "consumed")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unanswered_acio_tool_request_stays_pending_and_never_runs() {
    let (root, initial) = support::setup("shared");
    let executable = script(
        &root,
        "acio-timeout-worker.sh",
        &format!(
            "{EVENT_HEADER}emit session \"{{\\\"session_id\\\":\\\"$session_id\\\"}}\"\nemit tool '{{\"call_id\":\"call-1\",\"name\":\"file.write\",\"status\":\"requested\",\"needs_decision\":true,\"input\":{{\"path\":\"one.txt\"}}}}'\nIFS= read -r control\nemit tool '{{\"call_id\":\"call-1\",\"name\":\"file.write\",\"status\":\"denied\"}}'\nemit result '{{\"status\":\"failure\",\"summary\":\"no decision\"}}'\n"
        ),
    );
    let (run, mut child) = launch(&root, initial, &executable, "1");
    assert!(child.wait().unwrap().success());
    let report = workflow::load(&run).unwrap();
    assert_eq!(report.state, "waiting_tool_approval");
    assert_eq!(report.tasks[0].status, "waiting_approval");
    let id = report.tasks[0].tool_request.as_deref().unwrap();
    assert!(
        Ledger::open(&run.join("state.sqlite"))
            .unwrap()
            .receipt(id)
            .unwrap()
            .is_some_and(|receipt| receipt.status == "pending")
    );
    assert!(!root.join("repo/one.txt").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn mismatched_live_session_is_rejected_before_a_tool_approval_is_created() {
    let (root, initial) = support::setup("shared");
    let executable = script(
        &root,
        "acio-session-mismatch.sh",
        &format!(
            "{EVENT_HEADER}session_id=foreign-session\nemit session \"{{\\\"session_id\\\":\\\"$session_id\\\"}}\"\nemit tool '{{\"call_id\":\"call-1\",\"name\":\"file.write\",\"status\":\"requested\",\"needs_decision\":true,\"input\":{{\"path\":\"one.txt\"}}}}'\n"
        ),
    );
    let report = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(workflow::run(
            plan(initial, &executable),
            &root.join("repo"),
            &root.join("run"),
            20,
        ))
        .unwrap();
    assert_eq!(report.tasks[0].status, "failed");
    assert!(
        Ledger::open(&root.join("run/state.sqlite"))
            .unwrap()
            .requests()
            .unwrap()
            .is_empty(),
        "mismatched protocol session must not reach the human approval ledger"
    );
    std::fs::remove_dir_all(root).unwrap();
}
