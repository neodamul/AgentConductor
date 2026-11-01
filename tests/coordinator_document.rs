use agentconductor::{coordinator, protocol::Provider};

fn parse(reply: &str) -> Result<serde_json::Value, String> {
    let root = std::env::temp_dir().join(format!(
        "ac-coordinator-document-{}.jsonl",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let line = serde_json::json!({
        "type":"result",
        "subtype":"success",
        "is_error":false,
        "result":reply
    });
    std::fs::write(&root, line.to_string() + "\n").unwrap();
    let result = coordinator::read_document(Provider::Claude, &root, "phases");
    std::fs::remove_file(root).unwrap();
    result
}

#[test]
fn complete_fence_after_an_unfinished_draft_is_selected() {
    let valid = r#"{"slug":"ready","phases":[]}"#;
    let reply = format!("```json\n{{\"analysis\":\"unfinished draft\n```json\n{valid}\n```");
    assert_eq!(parse(&reply).unwrap()["slug"], "ready");
}

#[test]
fn coordinator_replies_up_to_256_kib_are_accepted() {
    let prefix = r#"{"phases":[],"pad":""#;
    let suffix = r#""}"#;
    let padding = 256 * 1024 - prefix.len() - suffix.len();
    let reply = format!("{prefix}{}{suffix}", "x".repeat(padding));
    assert_eq!(reply.len(), 256 * 1024);
    assert!(parse(&reply).is_ok());
}

#[test]
fn coordinator_replies_over_256_kib_are_rejected() {
    let reply = serde_json::json!({"phases":[],"pad":"x".repeat(257 * 1024)}).to_string();
    let error = parse(&reply).unwrap_err();
    assert!(error.contains("256 KiB"), "{error}");
}
