#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn reviewing_a_finished_run_does_not_take_the_execution_lock() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    let lock = run.join("coordinator.lock");
    std::fs::write(&lock, "review-lock-sentinel").unwrap();
    assert!(
        !agentconductor::workflow::ensure_availability_watcher(
            &run,
            std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"))
        )
        .unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(lock).unwrap(),
        "review-lock-sentinel",
        "read-only review must not compete with the approving child"
    );
    std::fs::remove_dir_all(root).unwrap();
}
