#![cfg(unix)]
//! GitHub #235: a coordinator reply that is not the requested JSON is asked for once more, with the reason.
use agentconductor::{coordinator, plan::AgentProfile};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

fn profile(script: &std::path::Path) -> AgentProfile {
    serde_json::from_value(serde_json::json!({
        "id":"coordinator","executable":script,"args":[],"parser":"claude","permission_mode":"read_only"
    }))
    .unwrap()
}

fn agent(root: &std::path::Path, replies: &[&str]) -> std::path::PathBuf {
    let script = root.join("coordinator.sh");
    // The n-th call prints the n-th reply (the last one repeats) and keeps every prompt it was given.
    let mut body = String::from(
        "#!/bin/sh\nd=\"$(dirname \"$0\")\"\nn=$(cat \"$d/count\" 2>/dev/null || echo 0)\nn=$((n+1))\necho $n > \"$d/count\"\ncat > \"$d/prompt.$n\"\ncase $n in\n",
    );
    for (i, reply) in replies.iter().enumerate() {
        body += &format!(
            "{})\necho '{{\"type\":\"result\",\"is_error\":false,\"result\":\"{reply}\"}}';;\n",
            i + 1
        );
    }
    body += &format!(
        "*)\necho '{{\"type\":\"result\",\"is_error\":false,\"result\":\"{}\"}}';;\nesac\n",
        replies.last().unwrap()
    );
    std::fs::write(&script, body).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

async fn ask(root: &std::path::Path, replies: &[&str]) -> Result<serde_json::Value, String> {
    std::fs::create_dir_all(root.join("project")).unwrap();
    let script = agent(root, replies);
    coordinator::run_json_within(
        &profile(&script),
        &root.join("project"),
        &root.join("run"),
        "# Document\n\nx\n\n# Task\n\ny\n\n# Format\n\nReturn {\"answer\": \"text\"}.\n",
        "answer",
        None,
        Duration::from_secs(20),
    )
    .await
}

#[tokio::test]
async fn a_malformed_reply_is_asked_for_again_once_with_the_reason() {
    let root = std::env::temp_dir().join(format!("ac-format-retry-{}", std::process::id()));
    let value = ask(&root, &["Sure, here you go.", r#"{\"answer\":\"hello\"}"#])
        .await
        .unwrap();
    assert_eq!(value["answer"], "hello");
    let second = std::fs::read_to_string(root.join("prompt.2")).unwrap();
    assert!(second.contains("# Previous reply") && second.contains("Sure, here you go."));
    assert_eq!(second.matches("\n# Format\n").count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn two_malformed_replies_fail_after_exactly_two_runs() {
    let root = std::env::temp_dir().join(format!("ac-format-retry-fail-{}", std::process::id()));
    assert!(ask(&root, &["nope"]).await.is_err());
    assert_eq!(
        std::fs::read_to_string(root.join("count")).unwrap().trim(),
        "2"
    );
    std::fs::remove_dir_all(root).unwrap();
}
