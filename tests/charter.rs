//! GitHub #149 / R-170 / DD-180: a run keeps its charter and the coordinator cannot drift
//! from it silently.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::workflow::{self, decisions, guardrail};

async fn failed_run(
    mode: &str,
    marker: Option<&std::path::Path>,
) -> (std::path::PathBuf, std::path::PathBuf, workflow::Report) {
    let (root, mut plan) = support::setup("shared");
    if mode != "drift" {
        plan.tasks.truncate(1);
    }
    plan.tasks[0].prompt = "extra.txt".into();
    plan.tasks[0].test_command = vec!["true".into()];
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.executable = support::amend_fixture().to_string_lossy().into_owned();
    coordinator.args = vec![mode.into()];
    if let Some(marker) = marker {
        coordinator.environment.insert(
            "AC_DRIFT_MARKER".into(),
            marker.to_string_lossy().into_owned(),
        );
    }
    let run = root.join("run");
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(&run),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    (root, run, report)
}

#[test]
fn drift_counts_the_paths_beyond_the_charter() {
    let (root, plan) = support::setup("shared");
    let charter = guardrail::Charter::from_plan(&plan);
    assert_eq!(charter.goal, plan.goal);
    assert_eq!(charter.tasks.len(), plan.tasks.len());
    assert_eq!(guardrail::scope_growth(&charter, &plan), 0);
    let mut wider = plan.clone();
    wider.tasks[0].paths.push("a.txt".into());
    wider.tasks[1].paths.push("b.txt".into());
    assert_eq!(guardrail::scope_growth(&charter, &wider), 2);
    wider.tasks[0].paths.retain(|p| p != "a.txt");
    assert_eq!(guardrail::scope_growth(&charter, &wider), 1);
    wider.tasks[0]
        .paths
        .extend((0..8).map(|i| format!("extra-{i}.txt")));
    assert_eq!(guardrail::scope_growth(&charter, &wider), 9);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_started_run_records_its_charter_and_shares_it_with_the_review() {
    let (root, run, failed) = failed_run("amend", None).await;
    let rows = decisions::list(&run).unwrap();
    let charter = rows
        .iter()
        .find(|r| r["kind"] == "charter")
        .expect("charter row");
    assert_eq!(charter["judgment"], "recorded");
    let repaired = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(
        repaired.state, "waiting_approval",
        "{:?}",
        repaired.review_error
    );
    let evidence = std::fs::read_to_string(run.join("review-1/evidence.json")).unwrap();
    assert!(evidence.contains("one.txt"), "{evidence}");
    let applied = decisions::list(&run).unwrap();
    let amendment = applied.iter().find(|r| r["kind"] == "amendment").unwrap();
    assert!(
        amendment["detail"].as_str().unwrap().contains("of 8"),
        "{amendment}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn widening_beyond_the_charter_stops_the_run_with_goal_drift() {
    let marker = std::env::temp_dir().join(format!(
        "ac-drift-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let _ = std::fs::remove_file(&marker);
    let (root, run, failed) = failed_run("drift", Some(&marker)).await;
    let stopped = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(stopped.state, "blocked", "{:?}", stopped.review_error);
    assert!(
        stopped
            .review_error
            .as_deref()
            .unwrap()
            .contains("goal_drift"),
        "{:?}",
        stopped.review_error
    );
    // The first five paths stay applied; the second amendment would exceed the per-run limit.
    assert_eq!(stopped.plan.tasks[0].paths.len(), 6);
    let rows = decisions::list(&run).unwrap();
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "amendment" && r["judgment"] == "escalated")
    );
    let _ = std::fs::remove_file(&marker);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn an_approved_scope_change_records_an_updated_charter() {
    let (root, run, _failed) = failed_run("amend", None).await;
    let paths = vec!["extra.txt".to_string()];
    let preview = workflow::recovery_preview(&run, "one.txt", &paths, None).unwrap();
    workflow::recovery_apply(
        &run,
        "one.txt",
        &paths,
        None,
        preview["digest"].as_str().unwrap(),
    )
    .unwrap();
    let rows = decisions::list(&run).unwrap();
    let updated = rows
        .iter()
        .find(|r| r["kind"] == "charter" && r["judgment"] == "updated");
    assert!(updated.is_some(), "{rows:?}");
    std::fs::remove_dir_all(root).unwrap();
}
