use agentconductor::{coordinator::read_document, protocol::Provider};
use serde_json::json;

#[test]
fn reply_at_the_256_kib_limit_is_accepted() {
    let prefix = r#"{"phases":[],"pad":""#;
    let suffix = r#""}"#;
    let size = 256 * 1024;
    let text = format!(
        "{prefix}{}{suffix}",
        "x".repeat(size - prefix.len() - suffix.len())
    );
    assert_eq!(text.len(), size);
    let path = std::env::temp_dir().join(format!("ac-boundary-{}.jsonl", std::process::id()));
    std::fs::write(
        &path,
        json!({"type":"result","is_error":false,"result":text}).to_string() + "\n",
    )
    .unwrap();
    assert!(read_document(Provider::Claude, &path, "phases").is_ok());
    std::fs::remove_file(path).unwrap();
}
