use agentconductor::{
    knowledge::{Lesson, Source},
    workflow,
};
use std::fs;
#[path = "support/workflow.rs"]
mod support;

fn draft() -> Lesson {
    Lesson {
        task: "one.txt".into(),
        context: "Authentication edge case".into(),
        failure: "Previous test omitted replay".into(),
        cause: "Only happy path verified".into(),
        response: "Add token replay assertions".into(),
        evidence: "Independent integration tests".into(),
        confidence: 0.8,
        revalidate: "Authentication changes".into(),
        sources: vec![Source {
            path: "base.txt".into(),
            sha256: workflow::digest(b"base\n"),
        }],
    }
}

#[tokio::test]
async fn exact_reviewed_lesson_is_assetized_and_changed_sources_are_excluded_from_handoff() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan.clone(), &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    let lesson = draft();
    let hash = workflow::digest(&serde_json::to_vec(&lesson).unwrap());
    assert!(workflow::knowledge_accept(&dir, &lesson, "wrong").is_err());
    let draft_file = root.join("lesson.json");
    fs::write(&draft_file, serde_json::to_vec_pretty(&lesson).unwrap()).unwrap();
    let digest = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("knowledge-digest")
        .arg(&draft_file)
        .output()
        .unwrap();
    assert!(digest.status.success());
    assert_eq!(String::from_utf8(digest.stdout).unwrap().trim(), hash);
    let accepted = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("knowledge-accept")
        .arg(&dir)
        .arg(&draft_file)
        .arg(&hash)
        .output()
        .unwrap();
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    let text = agentconductor::knowledge::context(&dir, &root.join("repo"), "one.txt").unwrap();
    assert!(text.contains("Add token replay assertions"));
    assert!(
        agentconductor::knowledge::context(&dir, &root.join("repo"), "two.txt")
            .unwrap()
            .is_empty()
    );
    assert!(dir.join("knowledge/wiki/index.md").exists());
    assert!(dir.join(format!("knowledge/wiki/{hash}.md")).exists());
    let log = fs::read_to_string(dir.join("knowledge/wiki/log.md")).unwrap();
    workflow::knowledge_accept(&dir, &lesson, &hash).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join("knowledge/wiki/log.md")).unwrap(),
        log
    );
    workflow::revise(&dir, "one.txt", "Apply the reviewed lesson").unwrap();
    let rerun = workflow::resume_due(&dir, i64::MAX, 5).await.unwrap();
    let prompt = fs::read_to_string(rerun.tasks[0].artifacts.join("prompt.txt")).unwrap();
    assert!(prompt.contains("Add token replay assertions"));
    let other = fs::read_to_string(rerun.tasks[1].artifacts.join("prompt.txt")).unwrap();
    assert!(!other.contains("Add token replay assertions"));
    fs::write(root.join("repo/base.txt"), "changed\n").unwrap();
    assert!(
        agentconductor::knowledge::context(&dir, &root.join("repo"), "one.txt")
            .unwrap()
            .is_empty()
    );
    let status = agentconductor::knowledge::status(&dir, &root.join("repo")).unwrap();
    assert_eq!(status[0]["status"], "stale");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn wiki_alias_failure_preserves_authoritative_review_and_never_writes_through_aliases() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    fs::create_dir(root.join("outside")).unwrap();
    std::os::unix::fs::symlink(root.join("outside"), dir.join("knowledge/wiki")).unwrap();
    let lesson = draft();
    let hash = workflow::digest(&serde_json::to_vec(&lesson).unwrap());
    let error = workflow::knowledge_accept(&dir, &lesson, &hash).unwrap_err();
    assert!(error.contains("saved in SQLite"));
    assert!(fs::read_dir(root.join("outside")).unwrap().next().is_none());
    assert_eq!(
        agentconductor::knowledge::status(&dir, &root.join("repo")).unwrap()[0]["status"],
        "fresh"
    );
    fs::remove_file(dir.join("knowledge/wiki")).unwrap();
    workflow::knowledge_accept(&dir, &lesson, &hash).unwrap();
    assert!(dir.join(format!("knowledge/wiki/{hash}.md")).exists());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn unsupported_sources_cannot_be_promoted_as_canonical_knowledge() {
    let (root, plan) = support::setup("shared");
    let dir = root.join("run");
    workflow::run(plan.clone(), &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    fs::write(root.join("repo/.env"), "synthetic secret").unwrap();
    for source in ["../outside.txt", ".env", ".git/config"] {
        let mut lesson = draft();
        lesson.sources[0].path = source.into();
        let hash = workflow::digest(&serde_json::to_vec(&lesson).unwrap());
        assert!(workflow::knowledge_accept(&dir, &lesson, &hash).is_err());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("repo/base.txt"), root.join("repo/alias.txt"))
            .unwrap();
        let mut lesson = draft();
        lesson.sources[0].path = "alias.txt".into();
        let hash = workflow::digest(&serde_json::to_vec(&lesson).unwrap());
        assert!(workflow::knowledge_accept(&dir, &lesson, &hash).is_err());
    }
    fs::remove_dir_all(root).unwrap();
}
