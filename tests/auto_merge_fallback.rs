//! #959: reviewed automatic fallbacks preserve opt-in; manual changes do not.
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{
    plan::{Fallback, Plan},
    workflow,
};
use std::{fs, path::PathBuf};
fn fallback(mode: &str) -> (PathBuf, Plan) {
    let (root, mut plan) = support::setup(mode);
    let mut backup = plan.agents[0].clone();
    backup.id = "company".into();
    backup.quota_group = Some("company-account".into());
    backup.args = vec!["task-preserve".into()];
    let mut later = backup.clone();
    later.id = "later".into();
    later.args = vec!["task".into()];
    plan.agents[0].args = vec!["quota-partial".into()];
    plan.agents[0].quota_group = Some("paid-account".into());
    plan.agents[0].fallback = Some(Fallback {
        agent: "company".into(),
        model: None,
    });
    plan.agents.extend([backup, later]);
    plan.tasks[1].agent = "later".into();
    plan.tasks[2].agent = "later".into();
    (root, plan)
}
#[tokio::test]
async fn reviewed_fallback_keeps_exact_auto_merge_consent_in_both_modes() {
    for mode in ["shared", "worktree"] {
        let (root, plan) = fallback(mode);
        let run = root.join("run");
        workflow::set_auto_merge(&run, true, &plan).unwrap();
        let report = workflow::run(plan, &root.join("repo"), &run, 10)
            .await
            .unwrap();
        assert_eq!(report.state, "waiting_approval");
        assert_eq!(report.plan.tasks[0].agent, "company");
        assert!(
            workflow::automatic_merge(&run).expect("reviewed fallback must not invalidate consent")
        );
        assert_eq!(workflow::load(&run).unwrap().state, "completed");
        let store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
        assert_eq!(store.document("approval").unwrap()["automatic"], true);
        assert!(
            !store
                .audit("automatic_merge_fallback_consent", 10)
                .unwrap()
                .is_empty()
        );
        drop(store);
        workflow::release_run_worktrees(&run);
        fs::remove_dir_all(root).unwrap();
    }
}
#[tokio::test]
async fn disabled_or_previously_mismatched_consent_is_not_reauthorized_by_fallback() {
    for tamper in [false, true] {
        let (root, plan) = fallback("shared");
        let run = root.join("run");
        workflow::set_auto_merge(&run, tamper, &plan).unwrap();
        if tamper {
            fs::write(
                run.with_extension("auto-merge.json"),
                r#"{"enabled":true,"plan_digest":"unapproved"}"#,
            )
            .unwrap();
        }
        workflow::run(plan, &root.join("repo"), &run, 10)
            .await
            .unwrap();
        let result = workflow::automatic_merge(&run);
        if tamper {
            assert!(result.is_err());
        } else {
            assert!(!result.unwrap());
        }
        let store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
        assert!(
            store
                .audit("automatic_merge_fallback_consent", 10)
                .unwrap()
                .is_empty()
        );
        drop(store);
        workflow::release_run_worktrees(&run);
        fs::remove_dir_all(root).unwrap();
    }
}
#[tokio::test]
async fn blocked_automatic_merge_worker_reports_a_reason_instead_of_success() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    workflow::set_auto_merge(&run, true, &plan).unwrap();
    let mut report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    report.review_round = 1;
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "blocked review",
        )
        .unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("phase-auto-merge")
        .arg(&run)
        .output()
        .unwrap();
    assert!(
        !result.status.success(),
        "blocked merge must surface an actionable job error"
    );
    assert!(!result.stderr.is_empty());
    assert_eq!(workflow::load(&run).unwrap().state, "waiting_approval");
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn manual_reassignment_does_not_inherit_automatic_fallback_consent() {
    let (root, mut plan) = support::setup("shared");
    let mut backup = plan.agents[0].clone();
    backup.id = "backup".into();
    plan.agents.push(backup);
    let run = root.join("run");
    workflow::set_auto_merge(&run, true, &plan).unwrap();
    workflow::run(plan.clone(), &root.join("repo"), &run, 10)
        .await
        .unwrap();
    let original = fs::read(run.with_extension("auto-merge.json")).unwrap();
    workflow::reassign(&run, &plan.tasks[0].id, "backup", None).unwrap();
    let repaired = workflow::resume_due(&run, i64::MAX, 10).await;
    assert!(
        repaired.is_ok(),
        "repair failed: {:?}",
        repaired.as_ref().err()
    );
    assert_eq!(workflow::load(&run).unwrap().state, "waiting_approval");
    assert_eq!(
        fs::read(run.with_extension("auto-merge.json")).unwrap(),
        original
    );
    assert!(workflow::automatic_merge(&run).is_err());
    fs::remove_dir_all(root).unwrap();
}
