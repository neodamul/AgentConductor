use agentconductor::ui::setup::Startup;
use serde_json::json;
use std::{path::Path, process::Command};

#[test]
fn oversized_question_is_rejected_without_starting_a_chat_job() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-startup-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(&root, &std::env::current_exe().unwrap()).unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let request = serde_json::to_vec(&json!({
        "kind":"chat", "revision":snapshot["chat_revision"], "text":"x".repeat(9 * 1024)
    }))
    .unwrap();
    let error = startup.dispatch(&request).unwrap_err();
    assert!(
        error.contains("question") || error.contains("질문"),
        "{error}"
    );
    assert!(startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["job"].is_null());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn chat_history_is_returned_on_demand_and_reset_persists_across_reopen() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-history-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let exe = std::env::current_exe().unwrap();
    let startup = Startup::open(&root, &exe).unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("startup.json")).unwrap()).unwrap();
    let history = json!([
        {"role":"operator","text":"What changed?"},
        {"role":"assistant","text":"The request diagnostics were added."}
    ]);
    saved["chat_result"] = json!("chat-a1b2c3.json");
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("mcp")).unwrap();
    std::fs::write(
        root.join("mcp/chat-input-a1b2c3.json"),
        serde_json::to_vec(&json!({
            "turns":[{"role":"operator","text":"What changed?"}],
            "language":"English"
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("mcp/chat-a1b2c3.json"),
        serde_json::to_vec(&json!({"answer":"The request diagnostics were added."})).unwrap(),
    )
    .unwrap();
    drop(startup);

    let startup = Startup::open(&root, &exe).unwrap();
    let snapshot = startup
        .dispatch(br#"{"kind":"snapshot","chat_history":true}"#)
        .unwrap();
    assert_eq!(snapshot["chat_history"], history);
    let save = serde_json::to_vec(&json!({
        "kind":"save",
        "revision":snapshot["revision"],
        "config":snapshot["config"]
    }))
    .unwrap();
    assert_eq!(startup.dispatch(&save).unwrap()["saved"], true);
    let snapshot = startup
        .dispatch(br#"{"kind":"snapshot","chat_history":true}"#)
        .unwrap();
    assert_eq!(snapshot["chat_history"], history);
    let reset = serde_json::to_vec(&json!({
        "kind":"chat_reset",
        "revision":snapshot["revision"]
    }))
    .unwrap();
    assert_eq!(startup.dispatch(&reset).unwrap()["reset"], true);
    drop(startup);

    let startup = Startup::open(&root, &exe).unwrap();
    let snapshot = startup
        .dispatch(br#"{"kind":"snapshot","chat_history":true}"#)
        .unwrap();
    assert_eq!(snapshot["chat_history"], json!([]));
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_legacy_chat_without_its_deleted_input_still_exposes_its_answer() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-legacy-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let exe = std::env::current_exe().unwrap();
    let startup = Startup::open(&root, &exe).unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("startup.json")).unwrap()).unwrap();
    saved["chat_result"] = json!("chat-a1b2c3.json");
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    std::fs::create_dir_all(root.join("mcp")).unwrap();
    std::fs::write(
        root.join("mcp/chat-a1b2c3.json"),
        br#"{"answer":"The earlier question's answer."}"#,
    )
    .unwrap();
    drop(startup);

    let startup = Startup::open(&root, &exe).unwrap();
    let snapshot = startup
        .dispatch(br#"{"kind":"snapshot","chat_history":true}"#)
        .unwrap();
    assert_eq!(
        snapshot["chat_history"],
        json!([{"role":"assistant","text":"The earlier question's answer."}])
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_chat_artifact_pointer_is_rejected() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-pointer-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let exe = std::env::current_exe().unwrap();
    let startup = Startup::open(&root, &exe).unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("startup.json")).unwrap()).unwrap();
    saved["chat_result"] = json!("chat-../../outside.json");
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    drop(startup);
    let startup = Startup::open(&root, &exe).unwrap();
    assert!(
        startup
            .dispatch(br#"{"kind":"snapshot","chat_history":true}"#)
            .is_err()
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn chat_revision_ignores_git_head_and_rejects_stale_workspace() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-revision-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let repo = root.join("repo");
    let proposal = root.join("proposal");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&proposal).unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
    ] {
        git(&repo, &args);
    }
    std::fs::write(repo.join("base.txt"), "base\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "base"]);

    let exe = std::env::current_exe().unwrap();
    let startup = Startup::open(&root, &exe).unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("startup.json")).unwrap()).unwrap();
    saved["config"]["repository"] = json!(repo);
    saved["proposal"] = json!({
        "workspace": repo,
        "directory": proposal,
        "repository_base": "base",
        "run": root.join("run"),
        "phase": "ready"
    });
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    drop(startup);

    let startup = Startup::open(&root, &exe).unwrap();
    let first = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let chat_revision = first["chat_revision"].as_str().unwrap().to_owned();
    std::fs::write(repo.join("next.txt"), "next\n").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "advance head"]);
    let next = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_ne!(first["revision"], next["revision"]);
    assert_eq!(next["chat_revision"], chat_revision);
    drop(startup);

    let mut saved: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("startup.json")).unwrap()).unwrap();
    saved["config"]["goal"] = json!("changed after the chat snapshot");
    std::fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let startup = Startup::open(&root, &exe).unwrap();
    let error = startup
        .dispatch(
            &serde_json::to_vec(&json!({
                "kind":"chat", "revision":chat_revision, "text":"hello"
            }))
            .unwrap(),
        )
        .unwrap_err();
    assert!(
        error.contains("changed") || error.contains("refresh") || error.contains("새로고침"),
        "{error}"
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
