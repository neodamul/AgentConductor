#![cfg(unix)]
//! GitHub #115: `chat::ask` runs the coordinator read-only and returns its JSON answer.
use agentconductor::chat::{Turn, ask};
use agentconductor::plan::AgentProfile;
use std::os::unix::fs::PermissionsExt;

fn profile(script: &std::path::Path) -> AgentProfile {
    serde_json::from_value(serde_json::json!({
        "id":"coordinator","executable":script,"args":[],"parser":"claude",
        "permission_mode":"read_only"
    }))
    .unwrap()
}

#[tokio::test]
async fn the_coordinator_answer_is_returned_and_the_prompt_reaches_it() {
    let root = std::env::temp_dir().join(format!("ac-chat-ask-{}", std::process::id()));
    std::fs::create_dir_all(root.join("project")).unwrap();
    let script = root.join("coordinator.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\ncat > \"$(dirname \"$0\")/prompt.seen\"\necho '{\"type\":\"result\",\"is_error\":false,\"result\":\"{\\\"answer\\\":\\\"Run npm start\\\"}\"}'\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let turns = [Turn {
        role: "operator".into(),
        text: "How do I run it?".into(),
    }];
    let answer = ask(
        &profile(&script),
        &root.join("project"),
        &root.join("chat-run"),
        &turns,
        "English",
    )
    .await
    .unwrap();
    assert_eq!(answer, "Run npm start");
    let seen = std::fs::read_to_string(root.join("prompt.seen")).unwrap();
    assert!(seen.contains("How do I run it?") && seen.contains("# Manual"));
    std::fs::remove_dir_all(root).unwrap();
}
