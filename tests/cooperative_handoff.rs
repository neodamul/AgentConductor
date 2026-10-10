#![cfg(unix)]
//! DD-43: a completed cooperative handoff continues automatically on the reviewed fallback
//! target after the relinquishing worker's processes are gone, preserving partial work.
use agentconductor::{cooperative, plan::Plan, state::Store, workflow};
use std::{
    process::Command,
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;

#[test]
fn completed_handoff_moves_to_the_reviewed_fallback_without_a_competing_writer() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let (release, exit_release) = (root.join("release"), root.join("exit-release"));
    let mut value = serde_json::to_value(plan).unwrap();
    let mut fallback = value["agents"][0].clone();
    fallback["args"] = serde_json::json!(["task-preserve"]);
    fallback["quota_group"] = "fallback-account".into();
    let company = &mut value["agents"][0];
    company["id"] = "company".into();
    company["executable"] = "/usr/bin/python3".into();
    company["args"] = serde_json::json!([
        format!(
            "{}/tests/support/cooperative_worker.py",
            env!("CARGO_MANIFEST_DIR")
        ),
        "{session_id}"
    ]);
    company["cooperative"] = "checkpoint_v1".into();
    company["quota_group"] = "company-account".into();
    company["fallback"] = serde_json::json!({"agent":"mock","model":null});
    company["environment"] =
        serde_json::json!({"MOCK_RELEASE": release, "MOCK_EXIT_RELEASE": exit_release});
    value["agents"].as_array_mut().unwrap().push(fallback);
    for task in value["tasks"].as_array_mut().unwrap() {
        task["agent"] = "company".into();
    }
    let plan: Plan = serde_json::from_value(value).expect("reviewed fallback plan");
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
    let deadline = Instant::now() + Duration::from_secs(30);
    let wait = |done: &dyn Fn() -> bool, child: &mut std::process::Child, what: &str| {
        while !done() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                panic!("{what} timeout; evidence {}", root.display());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    };
    wait(
        &|| (0..3).all(|i| run.join(format!("task-{i}/checkpoint-ready")).exists()),
        &mut child,
        "bootstrap",
    );
    let report = workflow::load(&run).unwrap();
    let company_session = report.tasks[0].session_id.clone();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    cooperative::issue(
        &mut store,
        cooperative::Issue {
            revision: workflow::revision(&report).unwrap(),
            task: "one.txt".into(),
            action: cooperative::Action::Handoff,
            instruction: "Relinquish at checkpoint".into(),
            issued_at_ms: now,
            expires_at_ms: now + 60_000,
        },
        now,
    )
    .unwrap();
    std::fs::write(&release, "continue").unwrap();
    wait(
        &|| run.join("task-0/checkpoint-relinquished").exists(),
        &mut child,
        "relinquishment",
    );
    // Acknowledged but still alive: no replacement writer may start yet.
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(workflow::load(&run).unwrap().plan.tasks[0].agent, "company");
    std::fs::write(&exit_release, "exit").unwrap();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "run timeout; {}", root.display());
        std::thread::sleep(Duration::from_millis(20));
    };
    let cli = std::fs::read_to_string(root.join("cli.log")).unwrap();
    assert!(status.success(), "{cli}");
    let report = workflow::load(&run).unwrap();
    assert_eq!(report.state, "waiting_approval", "{cli}");
    assert_eq!(report.plan.tasks[0].agent, "mock");
    assert_eq!(report.tasks[0].status, "completed");
    assert_ne!(report.tasks[0].session_id, company_session);
    assert_eq!(
        std::fs::read_to_string(report.tasks[0].cwd.join("one.txt")).unwrap(),
        "partial\nimplemented\n",
        "the reviewed target continues from the relinquished partial work"
    );
    assert!(report.tests.iter().all(|t| t.exit_code == Some(0)));
    assert!(workflow::merge(&run).is_err(), "merge still needs approval");
    std::fs::remove_dir_all(root).unwrap();
}
