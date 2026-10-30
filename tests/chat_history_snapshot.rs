#[path = "support/chat_history_snapshot.rs"]
mod snapshot;

use serde_json::{Value, json};
use std::{fs, path::PathBuf};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-snapshot-{name}-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(root.join("mcp")).unwrap();
    root
}

fn saved(result: &str) -> Value {
    json!({"chat_result":result})
}

#[test]
fn artifact_history_restores_question_and_answer() {
    let root = workspace("complete");
    fs::write(
        root.join("mcp/chat-input-a1b2.json"),
        br#"{"turns":[{"role":"operator","text":"Question"}]}"#,
    )
    .unwrap();
    fs::write(root.join("mcp/chat-a1b2.json"), br#"{"answer":"Answer"}"#).unwrap();

    assert_eq!(
        json!(snapshot::turns(&root, &saved("chat-a1b2.json")).unwrap()),
        json!([
            {"role":"operator","text":"Question"},
            {"role":"assistant","text":"Answer"}
        ])
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn artifact_history_keeps_the_question_while_answer_is_pending() {
    let root = workspace("pending");
    fs::write(
        root.join("mcp/chat-input-c3d4.json"),
        br#"{"turns":[{"role":"operator","text":"Still working?"}]}"#,
    )
    .unwrap();

    assert_eq!(
        json!(snapshot::turns(&root, &saved("chat-c3d4.json")).unwrap()),
        json!([{"role":"operator","text":"Still working?"}])
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn artifact_history_rejects_a_path_like_result_reference() {
    let root = workspace("invalid");
    assert!(snapshot::turns(&root, &saved("chat-../../outside.json")).is_none());
    fs::remove_dir_all(root).unwrap();
}
