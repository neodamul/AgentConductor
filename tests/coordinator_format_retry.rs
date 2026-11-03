#![cfg(unix)]
//! GitHub #235: request one corrected coordinator reply when its JSON shape is unusable.
use agentconductor::{coordinator, plan::AgentProfile};
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

const PROMPT: &str = "# Document\n\nEvidence\n\n# Task\n\nAnswer the question.\n\n# Format\n\nReturn {\"answer\":\"text\"}.\n";

fn coordinator_agent(root: &Path, replies: &[&str]) -> PathBuf {
    let script = root.join("coordinator.sh");
    let mut body = String::from(
        "#!/bin/sh\nd=\"$(dirname \"$0\")\"\nn=$(cat \"$d/count\" 2>/dev/null || echo 0)\nn=$((n+1))\necho $n > \"$d/count\"\ncat > \"$d/prompt.$n\"\ncase $n in\n",
    );
    for (index, reply) in replies.iter().enumerate() {
        body += &format!(
            "{})\necho '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"{reply}\"}}';;\n",
            index + 1
        );
    }
    body += &format!(
        "*)\necho '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"{}\"}}';;\nesac\n",
        replies.last().unwrap()
    );
    std::fs::write(&script, body).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script
}

async fn ask(root: &Path, replies: &[&str]) -> Result<serde_json::Value, String> {
    std::fs::create_dir_all(root.join("project")).unwrap();
    let profile: AgentProfile = serde_json::from_value(serde_json::json!({
        "id":"coordinator",
        "executable":coordinator_agent(root, replies),
        "args":[],
        "parser":"claude",
        "permission_mode":"read_only"
    }))
    .unwrap();
    coordinator::run_json_within(
        &profile,
        &root.join("project"),
        &root.join("run"),
        PROMPT,
        "answer",
        None,
        Duration::from_secs(20),
    )
    .await
}

fn fixture(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ac-format-retry-{}-{name}", std::process::id()))
}

#[tokio::test]
async fn one_malformed_reply_is_retried_with_the_parse_reason() {
    let root = fixture("valid");
    let answer = ask(&root, &["Sure, here you go.", r#"{\"answer\":\"hello\"}"#])
        .await
        .unwrap();

    assert_eq!(answer["answer"], "hello");
    assert_eq!(
        std::fs::read_to_string(root.join("count")).unwrap().trim(),
        "2"
    );
    let retry = std::fs::read_to_string(root.join("prompt.2")).unwrap();
    assert!(retry.contains("# Previous reply"));
    assert!(retry.contains("no valid JSON answer document"));
    assert_eq!(retry.matches("\n# Format\n").count(), 1);
    assert!(root.join("run/process-format-1/stdout.jsonl").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn two_malformed_replies_stop_after_exactly_two_runs() {
    let root = fixture("invalid");
    let error = ask(&root, &["nope"]).await.unwrap_err();

    assert!(error.contains("no valid JSON answer document"));
    assert_eq!(
        std::fs::read_to_string(root.join("count")).unwrap().trim(),
        "2"
    );
    std::fs::remove_dir_all(root).unwrap();
}
