#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{state::Store, workflow};

#[test]
fn killed_coordinator_cannot_recover_until_children_exit_and_keeps_native_ids() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["task-slow".into()];
    let dir = root.join("run");
    let plan_path = root.join("plan.json");
    let bytes = serde_json::to_vec(&plan).unwrap();
    std::fs::write(&plan_path, &bytes).unwrap();
    let mut coordinator = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("run")
        .arg(plan_path)
        .arg(root.join("repo"))
        .arg(&dir)
        .arg(workflow::digest(&bytes))
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let ready = || {
        (0..3).all(|i| {
            std::fs::metadata(dir.join(format!("task-{i}/process/stdout.jsonl")))
                .is_ok_and(|m| m.len() > 0)
        })
    };
    while !ready() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let initialized = ready();
    assert!(workflow::recover(&dir).is_err());
    coordinator.kill().unwrap();
    coordinator.wait().unwrap();
    assert!(initialized);
    assert!(
        workflow::recover(&dir).is_err(),
        "live children must block recovery"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let competing = runtime.block_on(workflow::run(
        plan.clone(),
        &root.join("repo"),
        &root.join("competing"),
        1,
    ));
    assert!(
        competing.is_err(),
        "crashed coordinator's children must also block a different run"
    );
    assert!(!root.join("competing").exists());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let report = loop {
        let recovered = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .arg("recover")
            .arg(&dir)
            .output()
            .unwrap();
        if recovered.status.success() {
            let output: serde_json::Value = serde_json::from_slice(&recovered.stdout).unwrap();
            assert_eq!(output["state"], "stopped");
            break workflow::load(&dir).unwrap();
        } else {
            assert!(
                std::time::Instant::now() < deadline,
                "recovery stayed blocked: {}",
                String::from_utf8_lossy(&recovered.stderr)
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    };
    assert_eq!(report.state, "stopped");
    assert!(
        report.tasks[0]
            .session_id
            .as_ref()
            .is_some_and(|id| id.starts_with("mock-") && id.ends_with("-one.txt"))
    );
    assert_eq!(report.tasks[0].attempt, 1);
    assert!(workflow::merge(&dir).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn recovery_requires_complete_ownership_evidence_and_never_runs_agents() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let mut report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    report.state = "running".into();
    // Simulate the crash window after spawn but before the scheduler checkpoint.
    report.tasks[0].status = "pending".into();
    report.tasks[0].attempt = 0;
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Simulated interrupted checkpoint",
        )
        .unwrap();
    let owner = report.tasks[0].artifacts.join("process/owner.json");
    let original = std::fs::read(&owner).unwrap();
    std::fs::remove_file(&owner).unwrap();
    assert!(workflow::recover(&dir).is_err());
    assert_eq!(workflow::load(&dir).unwrap().state, "running");
    let mut live: serde_json::Value = serde_json::from_slice(&original).unwrap();
    live["pid"] = std::process::id().into();
    live["process_group"] = std::process::id().into();
    std::fs::write(&owner, serde_json::to_vec(&live).unwrap()).unwrap();
    assert!(workflow::recover(&dir).is_err());
    std::fs::write(&owner, original).unwrap();
    let recovered = workflow::recover(&dir).unwrap();
    assert_eq!(recovered.state, "stopped");
    assert_eq!(recovered.tasks[0].status, "stopped");
    assert_eq!(recovered.tasks[0].attempt, 1);
    assert!(workflow::merge(&dir).is_err());
    let resumed = workflow::resume(&dir, 10).await.unwrap();
    assert_eq!(resumed.state, "waiting_approval");
    assert_eq!(resumed.tasks[0].attempt, 2);
    std::fs::remove_dir_all(root).unwrap();
}
