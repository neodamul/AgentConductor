//! GitHub #247 (DD-201): a complete fenced JSON block is found after an unfinished one, and the reply limit is 256 KiB.
use agentconductor::{coordinator::read_document, protocol::Provider};
use serde_json::{Value, json};

/// The document the coordinator printed as the Claude CLI's final `result` line.
fn read(reply: &str) -> Result<Value, String> {
    let path = std::env::temp_dir().join(format!(
        "ac-document-{}-{:?}.jsonl",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(
        &path,
        json!({"type":"result","is_error":false,"result":reply}).to_string() + "\n",
    )
    .unwrap();
    let result = read_document(Provider::Claude, &path, "phases");
    std::fs::remove_file(path).unwrap();
    result
}

const GOOD: &str =
    r#"{"slug":"demo","analysis":"complete","phases":[{"id":"01","name":"Plan","tasks":[]}]}"#;

#[test]
fn an_unfinished_draft_before_a_complete_document_is_skipped() {
    let reply = format!("```json\n{{\"analysis\":\"unfinished draft\n```json\n{GOOD}\n```");
    assert_eq!(read(&reply).unwrap()["slug"], "demo");
}

#[test]
fn the_first_of_several_valid_blocks_wins() {
    let second = GOOD.replace("demo", "other");
    let reply = format!("```json\n{GOOD}\n```\ntext\n```json\n{second}\n```");
    assert_eq!(read(&reply).unwrap()["slug"], "demo");
}

#[test]
fn a_block_without_the_field_is_skipped_for_the_next_one() {
    let reply = format!("```json\n{{\"note\":1}}\n```\n\n```json\n{GOOD}\n```");
    assert_eq!(read(&reply).unwrap()["slug"], "demo");
}

#[test]
fn malformed_blocks_only_still_fail() {
    let reply = "```json\n{\"phases\":\n```\n```json\n{oops\n```";
    let error = read(reply).unwrap_err();
    assert!(error.contains("contains no valid JSON"), "{error}");
}

#[test]
fn the_reply_limit_is_256_kib() {
    let padded = |bytes: usize| json!({"phases":[],"pad":"x".repeat(bytes)}).to_string();
    assert!(read(&padded(100 * 1024)).is_ok());
    let error = read(&padded(257 * 1024)).unwrap_err();
    assert!(error.contains("256 KiB"), "{error}");
}
