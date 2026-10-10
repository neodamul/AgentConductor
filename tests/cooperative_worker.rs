#![cfg(unix)]
use agentconductor::{cooperative, plan::Plan, state::Store, workflow};
use std::{
    process::Command,
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;

#[test]
fn cooperative_capability_requires_owned_custom_session_and_reserved_environment() {
    let (root, plan) = support::setup("shared");
    let mut value = serde_json::to_value(plan).unwrap();
    assert!(
        value["agents"][0].get("cooperative").is_none(),
        "disabled capability must preserve existing plan digests"
    );
    value["agents"][0]["cooperative"] = "checkpoint_v1".into();
    let mut p: Plan = serde_json::from_value(value).expect("explicit capability registration");
    assert!(p.validate().is_err(), "session argv binding required");
    p.agents[0].args.push("{session_id}".into());
    p.validate().unwrap();
    p.agents[0]
        .environment
        .insert("AC_CHECKPOINT_RUN_DIR".into(), "foreign".into());
    assert!(
        p.validate().is_err(),
        "bootstrap environment must be coordinator-owned"
    );
    p.agents[0].environment.clear();
    p.agents[0].parser = agentconductor::protocol::Provider::Codex;
    assert!(
        p.validate().is_err(),
        "provider-specific initial session contract not verified"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn compiled_worker_polls_before_and_during_work_and_reports_without_granting_merge() {
    for mode in ["shared", "worktree"] {
        checkpoint_case(mode, false, "rework");
    }
}

#[test]
fn conflicting_returned_session_cannot_replace_the_owned_checkpoint_identity() {
    checkpoint_case("shared", true, "rework");
}

#[test]
fn acknowledged_pause_and_handoff_cannot_become_completed_work_or_restart_automatically() {
    for action in ["pause", "handoff"] {
        checkpoint_case("shared", false, action);
    }
}

#[test]
fn pending_or_accepted_directive_at_clean_exit_blocks_completion() {
    for ack in ["pending", "accepted"] {
        checkpoint_case_with_ack("shared", false, "rework", ack);
    }
}

fn checkpoint_case(mode: &str, foreign_session: bool, action: &str) {
    checkpoint_case_with_ack(mode, foreign_session, action, "completed");
}

fn checkpoint_case_with_ack(mode: &str, foreign_session: bool, action: &str, ack_mode: &str) {
    let (root, plan) = support::setup(mode);
    let run = root.join("run");
    let release = root.join("release");
    let mut value = serde_json::to_value(plan).unwrap();
    value["agents"][0]["executable"] = "/usr/bin/python3".into();
    value["agents"][0]["args"] = serde_json::json!([
        format!(
            "{}/tests/support/cooperative_worker.py",
            env!("CARGO_MANIFEST_DIR")
        ),
        "{session_id}"
    ]);
    value["agents"][0]["cooperative"] = "checkpoint_v1".into();
    let exit_release = root.join("exit-release");
    value["agents"][0]["environment"] = serde_json::json!({"MOCK_RELEASE": release, "MOCK_EXIT_RELEASE":exit_release, "MOCK_ACK_MODE":ack_mode});
    if foreign_session {
        value["agents"][0]["environment"]["MOCK_BAD_SESSION"] = "1".into();
    }
    let plan: Plan = serde_json::from_value(value).expect("cooperative profile");
    plan.validate().unwrap();
    let file = root.join("plan.json");
    std::fs::write(&file, serde_json::to_vec(&plan).unwrap()).unwrap();
    let digest = workflow::digest(&serde_json::to_vec(&plan).unwrap());
    let log = std::fs::File::create(root.join("cli.log")).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("run")
        .arg(&file)
        .arg(root.join("repo"))
        .arg(&run)
        .arg(digest)
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while !(0..3).all(|i| {
        run.join(format!("task-{i}"))
            .join("checkpoint-ready")
            .exists()
    }) {
        if let Some(status) = child.try_wait().unwrap() {
            panic!(
                "early exit {status}: {}",
                std::fs::read_to_string(root.join("cli.log")).unwrap()
            );
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("bootstrap timeout; evidence {}", root.display());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let report = workflow::load(&run).unwrap();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let directive = cooperative::issue(
        &mut store,
        cooperative::Issue {
            revision: workflow::revision(&report).unwrap(),
            task: "one.txt".into(),
            action: serde_json::from_value(action.into()).unwrap(),
            instruction: if action == "rework" {
                "Use revised content"
            } else {
                "Relinquish at checkpoint"
            }
            .into(),
            issued_at_ms: now,
            expires_at_ms: now + 60_000,
        },
        now,
    )
    .unwrap();
    std::fs::write(&release, "continue").unwrap();
    if action != "rework" {
        loop {
            let latest = workflow::load(&run).unwrap();
            if run.join("task-0/checkpoint-relinquished").exists()
                && latest.tasks[1..].iter().all(|t| t.status == "completed")
            {
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("live relinquishment timeout; evidence {}", root.display());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let latest = workflow::load(&run).unwrap();
        assert_eq!(latest.tasks[0].status, "running");
        let owner: agentconductor::process::Ownership =
            serde_json::from_slice(&std::fs::read(run.join("task-0/process/owner.json")).unwrap())
                .unwrap();
        assert!(
            owner.ensure_quiescent().is_err(),
            "acknowledgment is not process absence"
        );
        assert!(workflow::reassign(&run, "one.txt", "mock", None).is_err());
        let further = cooperative::Issue {
            revision: workflow::revision(&latest).unwrap(),
            task: "one.txt".into(),
            action: cooperative::Action::Rework,
            instruction: "Further work".into(),
            issued_at_ms: now,
            expires_at_ms: now + 60_000,
        };
        assert!(
            cooperative::issue(&mut store, further, now)
                .unwrap_err()
                .contains("relinquished")
        );
        std::fs::write(&exit_release, "exit").unwrap();
    }
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(
                status.success(),
                !foreign_session,
                "{}",
                std::fs::read_to_string(root.join("cli.log")).unwrap()
            );
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("worker timeout; evidence {}", root.display());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let report = workflow::load(&run).unwrap();
    if foreign_session {
        assert_eq!(report.state, "failed");
        assert!(report.candidate.is_none());
        for task in &report.tasks {
            assert_eq!(task.status, "failed");
            assert_ne!(task.session_id.as_deref(), Some("foreign-session"));
            assert!(task.error.as_ref().unwrap().contains("another session"));
        }
        std::fs::remove_dir_all(root).unwrap();
        return;
    }
    if ack_mode != "completed" {
        assert_eq!(report.state, "blocked");
        assert_eq!(report.tasks[0].status, "blocked");
        assert!(report.candidate.is_none());
        assert!(report.tasks[0].error.as_ref().unwrap().contains("remains"));
        std::fs::remove_dir_all(root).unwrap();
        return;
    }
    if action != "rework" {
        assert_eq!(
            report.state, "stopped",
            "relinquishment is not task completion"
        );
        assert_eq!(report.tasks[0].status, "stopped");
        assert!(report.candidate.is_none());
        assert!(report.tests.is_empty());
        assert_eq!(
            std::fs::read_to_string(report.tasks[0].cwd.join("one.txt")).unwrap(),
            "partial\n"
        );
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let scheduled = runtime
            .block_on(workflow::resume_due(&run, i64::MAX, 10))
            .unwrap();
        assert_eq!(
            workflow::revision(&scheduled).unwrap(),
            workflow::revision(&report).unwrap()
        );
        assert!(workflow::merge(&run).is_err());
        let result = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .arg("resume")
            .arg(&run)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let resumed = workflow::load(&run).unwrap();
        assert_eq!(resumed.state, "waiting_approval");
        assert_eq!(resumed.tasks[0].attempt, 2);
        assert_eq!(resumed.tasks[0].session_id, report.tasks[0].session_id);
        assert!(cooperative::poll(&mut store, &directive.address, now + 1000).is_err());
        std::fs::remove_dir_all(root).unwrap();
        return;
    }
    assert_eq!(report.state, "waiting_approval");
    assert!(
        report
            .tasks
            .iter()
            .all(|t| t.status == "completed" && t.session_established)
    );
    assert_eq!(
        std::fs::read_to_string(report.tasks[0].cwd.join("one.txt")).unwrap(),
        "revised\n"
    );
    let saved = store
        .document(&format!("cooperative_directive/{}", directive.id))
        .unwrap();
    assert_eq!(saved["state"], "completed");
    for task in &report.tasks {
        let progress = store
            .document(&format!("cooperative_report/{}/1", task.id))
            .unwrap();
        assert_eq!(progress["sequence"], 2);
        assert_eq!(
            progress["session_id"],
            task.session_id.as_ref().unwrap().as_str()
        );
    }
    assert!(workflow::merge(&run).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
