#![cfg(unix)]
//! GitHub #122 (DD-172): guided recovery from an out-of-scope integration failure.
#[path = "support/workflow.rs"]
mod support;

use agentconductor::workflow;
use serde_json::json;

/// A one-task run whose worker writes extra.txt, outside its approved path one.txt.
async fn failed_run() -> (std::path::PathBuf, std::path::PathBuf) {
    let (root, mut plan) = support::setup("shared");
    plan.tasks.truncate(1);
    plan.tasks[0].prompt = "extra.txt".into();
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    assert_eq!(
        report.error.as_deref(),
        Some("out-of-scope change: extra.txt")
    );
    (root, dir)
}

#[tokio::test]
async fn the_failure_report_lists_offending_files_and_each_tasks_approved_paths() {
    let (root, dir) = failed_run().await;
    let report = workflow::scope_failure(&dir).unwrap();
    assert_eq!(report["offending"], json!(["extra.txt"]));
    assert_eq!(report["tasks"][0]["id"], "one.txt");
    assert_eq!(report["tasks"][0]["paths"], json!(["one.txt"]));
    assert!(report["note"].as_str().unwrap().contains("feedback"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn feedback_alone_never_widens_the_scope() {
    let (root, dir) = failed_run().await;
    workflow::revise(&dir, "one.txt", "approved, keep extra.txt").unwrap();
    let report = workflow::load(&dir).unwrap();
    assert_eq!(report.plan.tasks[0].paths, vec!["one.txt".to_string()]);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn an_approved_plan_change_reuses_the_finished_work_and_reaches_review() {
    let (root, dir) = failed_run().await;
    let test = Some(vec!["true".to_string()]);
    let preview =
        workflow::recovery_preview(&dir, "one.txt", &["extra.txt".into()], test.clone()).unwrap();
    assert_eq!(preview["previous_paths"], json!(["one.txt"]));
    assert_eq!(preview["test_command"], json!(["true"]));
    let digest = preview["digest"].as_str().unwrap().to_owned();
    assert!(
        workflow::recovery_apply(
            &dir,
            "one.txt",
            &["extra.txt".into()],
            test.clone(),
            "not-the-digest"
        )
        .is_err(),
        "an unreviewed proposal is refused"
    );
    assert_eq!(
        workflow::load(&dir).unwrap().plan.tasks[0].paths,
        vec!["one.txt".to_string()]
    );
    workflow::recovery_apply(&dir, "one.txt", &["extra.txt".into()], test, &digest).unwrap();
    let ready = workflow::load(&dir).unwrap();
    assert_eq!(ready.state, "stopped");
    assert!(ready.error.is_none() && ready.candidate.is_none());
    assert_eq!(ready.plan.tasks[0].paths, vec!["extra.txt".to_string()]);

    let report = workflow::resume(&dir, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert_eq!(
        report.tasks[0].attempt, 1,
        "the finished work was reused, not redone"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn overlapping_ownership_is_refused_in_the_preview() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks[0].prompt = "extra.txt".into();
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    let overlap = workflow::recovery_preview(&dir, "one.txt", &["two.txt".into()], None);
    assert!(overlap.is_err(), "two.txt belongs to another task");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn quarantined_files_restore_without_overwriting_and_keep_their_recovery_copies() {
    let (root, dir) = failed_run().await;
    let stray = dir.join("stray-files");
    std::fs::create_dir_all(stray.join("docs")).unwrap();
    std::fs::write(stray.join("docs/new.md"), "moved\n").unwrap();
    std::fs::write(stray.join("docs/taken.md"), "moved\n").unwrap();
    let repo = root.join("repo");
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/taken.md"), "the operator's own file\n").unwrap();
    let mut store = agentconductor::state::Store::open(dir.join("state.sqlite")).unwrap();
    store
        .record(
            "stray_files_moved",
            &json!({"paths":["docs/new.md","docs/taken.md"],"to":stray}),
            "test fixture",
        )
        .unwrap();
    drop(store);

    let failure = workflow::scope_failure(&dir).unwrap();
    assert_eq!(failure["stray"][0]["path"], "docs/new.md");
    let paths = ["extra.txt".to_string(), "docs".to_string()];
    let preview = workflow::recovery_preview(&dir, "one.txt", &paths, None).unwrap();
    let action = |path: &str| {
        preview["restore"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["path"] == path)
            .unwrap()["action"]
            .clone()
    };
    assert_eq!(action("docs/new.md"), "restore");
    assert_eq!(action("docs/taken.md"), "conflict");
    let digest = preview["digest"].as_str().unwrap();
    workflow::recovery_apply(&dir, "one.txt", &paths, None, digest).unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.join("docs/new.md")).unwrap(),
        "moved\n"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("docs/taken.md")).unwrap(),
        "the operator's own file\n",
        "a conflicting destination is never overwritten"
    );
    assert!(
        stray.join("docs/new.md").is_file(),
        "the recovery copy is kept"
    );
    std::fs::remove_dir_all(root).unwrap();
}
