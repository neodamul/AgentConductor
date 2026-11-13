//! #943: automation opts in to exact tested candidates and preserves every blocking gate.
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow;
use std::fs;
fn persist(dir: &std::path::Path, report: &workflow::Report) {
    let mut store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(report).unwrap(),
            "test gate",
        )
        .unwrap();
}

#[tokio::test]
async fn automatic_merge_is_default_off_and_never_bypasses_stop_or_evidence() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    let report = workflow::run(plan.clone(), &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(!workflow::automatic_merge(&dir).unwrap());
    workflow::set_auto_merge(&dir, true, &plan).unwrap();
    for state in [
        "failed",
        "blocked",
        "waiting_tool_approval",
        "stopped",
        "waiting_availability",
    ] {
        let mut gated = report.clone();
        gated.state = state.into();
        persist(&dir, &gated);
        assert!(!workflow::automatic_merge(&dir).unwrap(), "{state}");
    }
    persist(&dir, &report);
    fs::write(dir.join("stop.request"), "stop").unwrap();
    assert!(!workflow::automatic_merge(&dir).unwrap());
    fs::remove_file(dir.join("stop.request")).unwrap();
    fs::write(root.join("phase-chain.stop"), "stop").unwrap();
    assert!(!workflow::automatic_merge(&dir).unwrap());
    fs::remove_file(root.join("phase-chain.stop")).unwrap();
    let mut denied = report.clone();
    denied.tasks[0].error = Some("permission_denied".into());
    persist(&dir, &denied);
    assert!(!workflow::automatic_merge(&dir).unwrap());
    let mut review = report.clone();
    review.latest_review = Some(agentconductor::coordinator::ReviewProposal {
        verdict: agentconductor::coordinator::Verdict::Blocked,
        summary: "blocked".into(),
        evidence: vec![],
        revisions: vec![],
    });
    persist(&dir, &review);
    assert!(!workflow::automatic_merge(&dir).unwrap());

    let mut incomplete = report.clone();
    incomplete.tests.clear();
    persist(&dir, &incomplete);
    assert!(workflow::automatic_merge(&dir).is_err());
    persist(&dir, &report);
    assert!(workflow::automatic_merge(&dir).unwrap());
    assert_eq!(workflow::load(&dir).unwrap().state, "completed");
    let store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    assert_eq!(store.document("approval").unwrap()["automatic"], true);
    drop(store); // Windows requires the SQLite handle closed before deleting its files.
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn paused_chain_cannot_launch_a_new_phase_worker() {
    let (root, plan) = support::setup("shared");
    fs::write(root.join("phase-chain.stop"), "stop").unwrap();
    let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
        .await
        .unwrap();
    assert_eq!(report.state, "stopped");
    assert!(!root.join("repo/one.txt").exists());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn reviewed_resume_hands_automatic_merge_to_a_fresh_job() {
    let (root, plan) = support::setup("shared");
    fs::write(root.join("phase-chain.stop"), "stop").unwrap();
    let run = root.join("run");
    let report = workflow::run(plan.clone(), &root.join("repo"), &run, 10)
        .await
        .unwrap();
    workflow::set_auto_merge(&run, true, &plan).unwrap();
    fs::remove_file(root.join("phase-chain.stop")).unwrap();
    let resumed = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("ui-action")
        .arg(&run)
        .arg(workflow::revision(&report).unwrap())
        .arg("ui-resume")
        .arg(workflow::stop_digest(&run).unwrap())
        .output()
        .unwrap();
    assert!(
        resumed.status.success(),
        "{}",
        String::from_utf8_lossy(&resumed.stderr)
    );
    assert_eq!(workflow::load(&run).unwrap().state, "waiting_approval");
    let merged = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("phase-auto-merge")
        .arg(&run)
        .output()
        .unwrap();
    assert!(
        merged.status.success(),
        "{}",
        String::from_utf8_lossy(&merged.stderr)
    );
    assert_eq!(workflow::load(&run).unwrap().state, "completed");
    fs::remove_dir_all(root).unwrap();
}
