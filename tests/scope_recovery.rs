#[path = "support/workflow.rs"]
mod support;

use agentconductor::{state::Store, workflow};
use serde_json::json;
use std::path::PathBuf;

async fn failed_run() -> (PathBuf, PathBuf) {
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
async fn failure_details_keep_scope_and_explain_quarantined_paths() {
    let (root, dir) = failed_run().await;
    let stray = dir.join("stray-files/docs/new.md");
    std::fs::create_dir_all(stray.parent().unwrap()).unwrap();
    std::fs::write(&stray, "preserved\n").unwrap();
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    store
        .record(
            "stray_files_moved",
            &json!({"paths":["docs/new.md"]}),
            "outside approved task scope",
        )
        .unwrap();
    drop(store);

    let failure = workflow::scope_failure(&dir).unwrap();
    assert_eq!(failure["offending"], json!(["extra.txt"]));
    assert_eq!(failure["tasks"][0]["paths"], json!(["one.txt"]));
    assert_eq!(failure["stray"][0]["path"], "docs/new.md");
    assert!(failure["note"].as_str().unwrap().contains("Feedback"));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn feedback_alone_does_not_widen_scope() {
    let (root, dir) = failed_run().await;
    workflow::revise(&dir, "one.txt", "also use extra.txt").unwrap();
    assert_eq!(
        workflow::load(&dir).unwrap().plan.tasks[0].paths,
        vec!["one.txt".to_string()]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn only_a_fresh_review_can_apply_and_completed_work_is_reused() {
    let (root, dir) = failed_run().await;
    let paths = ["extra.txt".to_string()];
    let test = Some(vec![
        support::fixture().to_string_lossy().into_owned(),
        "echo".into(),
    ]);
    let preview = workflow::recovery_preview(&dir, "one.txt", &paths, test.clone()).unwrap();
    let digest = preview["digest"].as_str().unwrap().to_owned();
    let completed_attempt = workflow::load(&dir).unwrap().tasks[0].attempt;
    assert!(workflow::recovery_apply(&dir, "one.txt", &paths, test.clone(), "stale").is_err());
    assert_eq!(
        workflow::load(&dir).unwrap().plan.tasks[0].paths,
        vec!["one.txt"]
    );

    workflow::recovery_apply(&dir, "one.txt", &paths, test, &digest).unwrap();
    let stopped = workflow::load(&dir).unwrap();
    assert_eq!(stopped.state, "stopped");
    assert!(stopped.error.is_none() && stopped.candidate.is_none());
    assert_eq!(stopped.plan.tasks[0].paths, paths);
    assert_eq!(stopped.tasks[0].status, "completed");
    let resumed = workflow::resume(&dir, 10).await.unwrap();
    assert_eq!(resumed.state, "waiting_approval", "{:?}", resumed.error);
    assert_eq!(resumed.tasks[0].attempt, completed_attempt);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn preview_refuses_paths_owned_by_another_task() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks[0].prompt = "extra.txt".into();
    let dir = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    assert!(workflow::recovery_preview(&dir, "one.txt", &["two.txt".into()], None).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn restoration_never_overwrites_and_keeps_quarantine_copies() {
    let (root, dir) = failed_run().await;
    let repo = root.join("repo");
    let stray = dir.join("stray-files/docs");
    std::fs::create_dir_all(&stray).unwrap();
    std::fs::write(stray.join("new.md"), "moved\n").unwrap();
    std::fs::write(stray.join("taken.md"), "moved\n").unwrap();
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/taken.md"), "human file\n").unwrap();
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    store
        .record(
            "stray_files_moved",
            &json!({"paths":["docs/new.md","docs/taken.md"]}),
            "outside approved task scope",
        )
        .unwrap();
    drop(store);

    let paths = ["extra.txt".into(), "docs".into()];
    let preview = workflow::recovery_preview(&dir, "one.txt", &paths, None).unwrap();
    let action = |path: &str| {
        preview["restore"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["path"] == path)
            .unwrap()["action"]
            .clone()
    };
    assert_eq!(action("docs/new.md"), "restore");
    assert_eq!(action("docs/taken.md"), "conflict");
    workflow::recovery_apply(
        &dir,
        "one.txt",
        &paths,
        None,
        preview["digest"].as_str().unwrap(),
    )
    .unwrap();
    assert_eq!(std::fs::read(repo.join("docs/new.md")).unwrap(), b"moved\n");
    assert_eq!(
        std::fs::read(repo.join("docs/taken.md")).unwrap(),
        b"human file\n"
    );
    assert!(stray.join("new.md").is_file());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn preview_refuses_a_symlink_destination_inside_the_workspace() {
    #[cfg(not(unix))]
    return;

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let (root, dir) = failed_run().await;
        let repo = root.join("repo");
        let outside = root.join("outside");
        let stray = dir.join("stray-files/docs");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::create_dir_all(&stray).unwrap();
        std::fs::write(stray.join("new.md"), "moved\n").unwrap();
        symlink(&outside, repo.join("docs")).unwrap();
        let mut store = Store::open(dir.join("state.sqlite")).unwrap();
        store
            .record(
                "stray_files_moved",
                &json!({"paths":["docs/new.md"]}),
                "outside approved task scope",
            )
            .unwrap();
        drop(store);

        assert!(
            workflow::recovery_preview(&dir, "one.txt", &["extra.txt".into(), "docs".into()], None)
                .is_err()
        );
        assert!(!outside.join("new.md").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn failure_report_refuses_parent_traversal_in_quarantine_audit() {
    let (root, dir) = failed_run().await;
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    store
        .record(
            "stray_files_moved",
            &json!({"paths":["../outside.md"]}),
            "corrupt fixture",
        )
        .unwrap();
    drop(store);
    assert!(workflow::scope_failure(&dir).is_err());
    assert!(!root.join("outside.md").exists());
    std::fs::remove_dir_all(root).unwrap();
}
