use agentconductor::{cooperative, plan::Plan, state::Store, workflow};
use std::{
    process::Command,
    time::{Duration, Instant},
};
#[path = "workflow.rs"]
pub mod support;

pub fn checkpoint_case_with_crash(
    mode: &str,
    foreign_session: bool,
    action: &str,
    ack_mode: &str,
    crash: bool,
) {
    let (root, plan) = support::setup(mode);
    let run = root.join("run");
    let release = root.join("release");
    let mut value = serde_json::to_value(plan).unwrap();
    value["agents"][0]["executable"] = env!("CARGO_BIN_EXE_test_agent_fixture").into();
    value["agents"][0]["args"] = serde_json::json!(["cooperative-worker", "{session_id}"]);
    value["agents"][0]["cooperative"] = "checkpoint_v1".into();
    let exit_release = root.join("exit-release");
    value["agents"][0]["environment"] = serde_json::json!({"MOCK_RELEASE": release, "MOCK_EXIT_RELEASE":exit_release, "MOCK_ACK_MODE":ack_mode});
    if crash && action == "rework" {
        value["agents"][0]["environment"]["MOCK_REQUIRE_CONTINUATION"] = "1".into();
    }
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
    }) || !workflow::load(&run).is_ok_and(|report| {
        report
            .tasks
            .iter()
            .all(|task| task.status == "running" && task.attempt > 0 && task.session_id.is_some())
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
    if action != "rework" || crash {
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
                .contains(if ack_mode == "completed" {
                    "relinquished"
                } else {
                    "outstanding"
                })
        );
        if crash {
            child.kill().unwrap();
            child.wait().unwrap();
            assert!(
                workflow::recover(&run).is_err(),
                "acknowledged live worker must prevent recovery after coordinator exit"
            );
            assert!(workflow::reassign(&run, "one.txt", "mock", None).is_err());
            assert_eq!(
                workflow::load(&run).unwrap().tasks[0].session_id,
                latest.tasks[0].session_id
            );
        }
        std::fs::write(&exit_release, "exit").unwrap();
    }
    if crash {
        loop {
            if workflow::recover(&run).is_ok() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "recovery timeout; evidence {}",
                root.display()
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    } else {
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
    }
    // Reap explicitly on both normal and killed-coordinator paths; wait is idempotent.
    child.wait().unwrap();
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
    if ack_mode != "completed" && !crash {
        assert_eq!(report.state, "blocked");
        assert_eq!(report.tasks[0].status, "blocked");
        assert!(report.candidate.is_none());
        assert!(report.tasks[0].error.as_ref().unwrap().contains("remains"));
        std::fs::remove_dir_all(root).unwrap();
        return;
    }
    if action != "rework" || crash {
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
        if action == "rework" {
            assert_eq!(
                resumed.tasks[0]
                    .checkpoint_context
                    .as_ref()
                    .unwrap()
                    .instruction,
                "Use revised content"
            );
            assert_eq!(
                std::fs::read_to_string(resumed.tasks[0].cwd.join("one.txt")).unwrap(),
                "revised\n"
            );
        }
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
