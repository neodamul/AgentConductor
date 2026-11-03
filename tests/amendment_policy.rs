#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;

use agentconductor::{
    state::Store,
    workflow::{self, decisions, guardrail},
};
use serde_json::json;

#[test]
fn amendments_reuse_plan_scope_and_dependency_validation() {
    let (root, plan) = support::setup("shared");
    let safe = guardrail::amend(&plan, 0, &["docs/new/file.md".into()], &[]).unwrap();
    assert!(safe.risk.is_none());
    assert!(
        safe.plan.tasks[0]
            .paths
            .contains(&"docs/new/file.md".into())
    );
    assert!(
        guardrail::amend(&plan, 0, &["src".into()], &[])
            .unwrap()
            .risk
            .is_some()
    );
    assert!(
        guardrail::amend(&plan, 0, &["assets.v1/".into()], &[])
            .unwrap()
            .risk
            .is_some()
    );
    assert!(
        guardrail::amend(&plan, 0, &[".env".into()], &[])
            .unwrap()
            .risk
            .is_some()
    );
    for path in ["../escape", "/absolute", ".git/config", "two.txt", ""] {
        assert!(
            guardrail::amend(&plan, 0, &[path.into()], &[]).is_err(),
            "{path}"
        );
    }
    assert!(guardrail::amend(&plan, 99, &["safe.txt".into()], &[]).is_err());
    assert!(guardrail::amend(&plan, 0, &[], &["missing".into()]).is_err());
    let mut existing_dependency = plan.clone();
    existing_dependency.tasks[0]
        .depends_on
        .push("two.txt".into());
    assert!(guardrail::amend(&existing_dependency, 0, &[], &["two.txt".into()]).is_err());
    let cyclic = guardrail::amend(&plan, 0, &[], &["two.txt".into()]).unwrap();
    assert!(guardrail::amend(&cyclic.plan, 1, &[], &["one.txt".into()]).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn amendment_application_rechecks_payload_bounds() {
    let (root, plan) = support::setup("shared");
    let paths = (0..9)
        .map(|n| format!("src/module{n}.rs"))
        .collect::<Vec<_>>();
    assert!(guardrail::amend(&plan, 0, &paths, &[]).is_err());
    assert!(guardrail::amend(&plan, 0, &["x".repeat(1025)], &[]).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

async fn out_of_scope(mode: &str) -> (std::path::PathBuf, std::path::PathBuf, workflow::Report) {
    let (root, mut plan) = support::setup("shared");
    plan.tasks.truncate(if mode == "overlap" { 2 } else { 1 });
    plan.tasks[0].prompt = "extra.txt".into();
    plan.tasks[0].test_command = vec!["true".into()];
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.executable = support::amend_fixture().to_string_lossy().into_owned();
    coordinator.args = vec![mode.into()];
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

async fn failed_test_command() -> (std::path::PathBuf, std::path::PathBuf, workflow::Report) {
    let (root, mut plan) = support::setup("shared");
    plan.tasks.truncate(1);
    plan.tasks[0].test_command = vec!["false".into()];
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.executable = support::amend_fixture().to_string_lossy().into_owned();
    coordinator.args = vec!["test-command".into()];
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

#[tokio::test]
async fn test_command_amendment_is_validated_recorded_and_undoable() {
    let (root, run, failed) = failed_test_command().await;
    assert!(failed.state == "failed", "{:?}", failed.error);
    let revised = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(
        revised.state, "waiting_approval",
        "{:?}",
        revised.review_error
    );
    assert_eq!(revised.plan.tasks[0].test_command, ["true"]);
    assert!(
        agentconductor::workflow::guardrail::charter_drift(&revised.plan, revised.charter.as_ref())
            .is_empty()
    );
    let applied = decisions::list(&run)
        .unwrap()
        .into_iter()
        .find(|row| row["kind"] == "amendment")
        .unwrap();
    assert_eq!(applied["judgment"], "applied");
    assert!(
        applied["detail"]
            .as_str()
            .unwrap()
            .contains("replace_test_command")
    );
    let id = applied["id"].as_str().unwrap();
    workflow::amendment_undo(&run, id).unwrap();
    let undone = workflow::load(&run).unwrap();
    assert_eq!(undone.plan.tasks[0].test_command, ["false"]);
    assert_eq!(undone.state, "stopped");
    assert!(
        agentconductor::workflow::guardrail::charter_drift(&undone.plan, undone.charter.as_ref())
            .is_empty()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn safe_amendment_is_audited_undoable_and_revokes_candidate() {
    let (root, run, failed) = out_of_scope("safe").await;
    assert!(workflow::repairable(&failed));
    let repaired = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(
        repaired.state, "waiting_approval",
        "{:?}",
        repaired.review_error
    );
    assert!(repaired.plan.tasks[0].paths.contains(&"extra.txt".into()));
    assert!(
        agentconductor::workflow::guardrail::charter_drift(
            &repaired.plan,
            repaired.charter.as_ref()
        )
        .is_empty()
    );
    let applied = decisions::list(&run)
        .unwrap()
        .into_iter()
        .find(|row| row["kind"] == "amendment")
        .unwrap();
    assert_eq!(applied["judgment"], "applied");
    let id = applied["id"].as_str().unwrap();
    workflow::amendment_undo(&run, id).unwrap();
    let undone = workflow::load(&run).unwrap();
    assert_eq!(undone.state, "stopped");
    assert_eq!(undone.plan.tasks[0].paths, ["one.txt"]);
    assert!(
        agentconductor::workflow::guardrail::charter_drift(&undone.plan, undone.charter.as_ref())
            .is_empty()
    );
    assert!(undone.candidate.is_none());
    assert!(workflow::amendment_undo(&run, id).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn risky_amendment_stops_for_a_person_without_expanding_scope() {
    for (mode, reason) in [
        ("directory", "whole top-level directory"),
        ("secret", "sensitive path"),
        ("overlap", "overlapping"),
    ] {
        let (root, run, failed) = out_of_scope(mode).await;
        let blocked = workflow::repair(&run, 10, failed).await.unwrap();
        assert_eq!(
            blocked.state, "blocked",
            "{mode}: {:?}",
            blocked.review_error
        );
        assert_eq!(blocked.plan.tasks[0].paths, ["one.txt"]);
        assert!(blocked.review_error.as_deref().unwrap().contains(reason));
        let row = decisions::list(&run)
            .unwrap()
            .into_iter()
            .find(|row| row["kind"] == "amendment")
            .unwrap();
        assert_eq!(row["judgment"], "escalated");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn amendment_budget_is_bounded_per_run() {
    let (root, run, failed) = out_of_scope("safe").await;
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    for n in 0..5 {
        store
            .record(
                "amendment",
                &json!({"id":format!("old-{n}"),"status":"applied","task":"other","changes":[]}),
                "prior amendment",
            )
            .unwrap();
    }
    drop(store);
    let blocked = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(blocked.state, "blocked");
    assert!(blocked.review_error.as_deref().unwrap().contains("budget"));
    assert_eq!(blocked.plan.tasks[0].paths, ["one.txt"]);
    std::fs::remove_dir_all(root).unwrap();
}
