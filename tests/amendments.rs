//! GitHub #149 / R-169 / DD-179: the coordinator amends the plan within limits, every
//! amendment is recorded and can be undone, and everything else waits for a person.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{
    plan::Plan,
    state::Store,
    workflow::{self, decisions, guardrail},
};
use serde_json::json;

fn plan() -> (std::path::PathBuf, Plan) {
    let (root, plan) = support::setup("shared");
    (root, plan)
}

fn paths(plan: &Plan, task: usize) -> Vec<String> {
    plan.tasks[task].paths.clone()
}

#[test]
fn the_validator_accepts_low_risk_paths_and_dependencies() {
    let (root, plan) = plan();
    for path in ["extra.txt", "docs/new/file.md"] {
        let amended = guardrail::amend(&plan, 0, &[path.into()], &[]).unwrap();
        assert!(amended.risk.is_none(), "{path}: {:?}", amended.risk);
        assert!(paths(&amended.plan, 0).contains(&path.to_string()));
        assert_eq!(amended.changes[0]["kind"], "add_path");
    }
    let amended = guardrail::amend(&plan, 0, &[], &["two.txt".into()]).unwrap();
    assert_eq!(
        amended.plan.tasks[0].depends_on,
        vec!["two.txt".to_string()]
    );
    assert_eq!(amended.changes[0]["kind"], "add_dependency");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_validator_marks_broad_or_sensitive_paths_high_risk() {
    let (root, plan) = plan();
    for path in [
        "src",
        ".env",
        "secrets/token.txt",
        "keys/id_rsa",
        "deploy.pem",
        ".ssh/config",
        ".aws/credentials",
    ] {
        let amended = guardrail::amend(&plan, 0, &[path.into()], &[]).unwrap();
        assert!(amended.risk.is_some(), "{path} needs a person");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_validator_refuses_unsafe_overlapping_and_cyclic_changes() {
    let (root, plan) = plan();
    for path in ["../x", "/abs", ".git/config", "two.txt", "*.rs", ""] {
        assert!(
            guardrail::amend(&plan, 0, &[path.into()], &[]).is_err(),
            "{path}"
        );
    }
    assert!(
        guardrail::amend(&plan, 0, &[], &["one.txt".into()]).is_err(),
        "self dependency"
    );
    assert!(
        guardrail::amend(&plan, 0, &[], &["nope".into()]).is_err(),
        "unknown dependency"
    );
    let first = guardrail::amend(&plan, 0, &[], &["two.txt".into()]).unwrap();
    assert!(
        guardrail::amend(&first.plan, 1, &[], &["one.txt".into()]).is_err(),
        "cycle"
    );
    assert!(
        guardrail::amend(&plan, 0, &[], &[]).is_err(),
        "nothing to amend"
    );
    std::fs::remove_dir_all(root).unwrap();
}

async fn failed_run(mode: &str) -> (std::path::PathBuf, std::path::PathBuf, workflow::Report) {
    let (root, mut plan) = support::setup("shared");
    plan.tasks.truncate(if mode == "overlap" { 2 } else { 1 });
    plan.tasks[0].prompt = "extra.txt".into();
    // The agent writes extra.txt, so the fixture test for one.txt would fail for another reason.
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

#[tokio::test]
async fn a_low_risk_amendment_continues_the_run_and_can_be_undone() {
    let (root, run, failed) = failed_run("amend").await;
    assert_eq!(failed.state, "failed");
    assert!(
        failed
            .error
            .as_deref()
            .unwrap()
            .starts_with("out-of-scope change:")
    );
    assert!(
        workflow::repairable(&failed),
        "the coordinator decides scope escapes first"
    );

    let repaired = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(
        repaired.state, "waiting_approval",
        "{:?} {:?}",
        repaired.error, repaired.review_error
    );
    assert!(
        repaired.plan.tasks[0]
            .paths
            .contains(&"extra.txt".to_string())
    );

    let rows = decisions::list(&run).unwrap();
    let applied = rows
        .iter()
        .find(|r| r["kind"] == "amendment")
        .expect("amendment row");
    assert_eq!(applied["judgment"], "applied");
    assert_eq!(applied["task"], "one.txt");
    assert!(applied["detail"].as_str().unwrap().contains("extra.txt"));
    let id = applied["id"].as_str().unwrap().to_string();
    assert_eq!(applied["undone"], false);

    // Undo needs the run to wait for the operator, and restores exactly the earlier scope.
    workflow::amendment_undo(&run, &id).unwrap();
    let undone = workflow::load(&run).unwrap();
    assert_eq!(undone.plan.tasks[0].paths, vec!["one.txt".to_string()]);
    assert_eq!(undone.state, "stopped");
    assert!(undone.candidate.is_none());
    let rows = decisions::list(&run).unwrap();
    assert!(
        rows.iter()
            .any(|r| r["kind"] == "amendment" && r["judgment"] == "undone")
    );
    assert_eq!(
        rows.iter().find(|r| r["judgment"] == "applied").unwrap()["undone"],
        true
    );
    assert!(workflow::amendment_undo(&run, &id).is_err(), "only once");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn broad_sensitive_and_overlapping_amendments_wait_for_a_person() {
    for (mode, reason) in [
        ("dir", "whole top-level directory"),
        ("secret", "sensitive path"),
        ("overlap", "overlapping"),
    ] {
        let (root, run, failed) = failed_run(mode).await;
        let before = failed.plan.tasks[0].paths.clone();
        let stopped = workflow::repair(&run, 10, failed).await.unwrap();
        assert_eq!(
            stopped.state, "blocked",
            "{mode}: {:?}",
            stopped.review_error
        );
        assert_eq!(
            stopped.plan.tasks[0].paths, before,
            "{mode}: the plan is unchanged"
        );
        assert!(
            stopped.review_error.as_deref().unwrap().contains(reason),
            "{mode}: {:?}",
            stopped.review_error
        );
        let rows = decisions::list(&run).unwrap();
        let row = rows.iter().find(|r| r["kind"] == "amendment").expect("row");
        assert_eq!(row["judgment"], "escalated", "{mode}");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn the_sixth_amendment_of_a_run_waits_for_a_person() {
    let (root, run, failed) = failed_run("amend").await;
    {
        let mut store = Store::open(run.join("state.sqlite")).unwrap();
        for n in 0..5 {
            store
                .record("amendment", &json!({"id":format!("am-old-{n}"),"status":"applied","task":"one.txt","changes":[],"previous":{"paths":["one.txt"],"depends_on":[]}}), "earlier amendment")
                .unwrap();
        }
    }
    let stopped = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(stopped.state, "blocked");
    assert!(
        stopped.review_error.as_deref().unwrap().contains("budget"),
        "{:?}",
        stopped.review_error
    );
    assert_eq!(stopped.plan.tasks[0].paths, vec!["one.txt".to_string()]);
    std::fs::remove_dir_all(root).unwrap();
}

/// The room's Undo button goes through the same request the page sends.
#[tokio::test]
async fn the_room_undoes_an_amendment() {
    let (root, run, failed) = failed_run("amend").await;
    let repaired = workflow::repair(&run, 10, failed).await.unwrap();
    assert_eq!(repaired.state, "waiting_approval");
    let id = decisions::list(&run)
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "amendment")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::open(&run, exe).unwrap();
    let ask = |v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    assert_eq!(
        ask(json!({"kind":"undo_amendment","id":id})).unwrap()["undone"],
        id
    );
    assert_eq!(
        workflow::load(&run).unwrap().plan.tasks[0].paths,
        vec!["one.txt".to_string()]
    );
    assert!(ask(json!({"kind":"undo_amendment","id":id})).is_err());
    assert!(ask(json!({"kind":"undo_amendment","id":"am-unknown"})).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
