//! GitHub #185: the coordinator conversation is rebuilt from the stored artifacts of the latest chat.
use agentconductor::{chat::history, mcp::store};
use serde_json::json;

#[test]
fn the_history_is_the_stored_turns_plus_the_answer_when_there_is_one() {
    let root = std::env::temp_dir().join(format!("ac-chat-history-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    // Artifacts open without following links, so the temporary folder must be canonical.
    let root = root.canonicalize().unwrap();
    assert!(
        history(&root, "chat-none.json").is_empty(),
        "nothing stored yet"
    );
    let turns = json!([{"role":"operator","text":"What was built?"}]);
    store(
        &root,
        "chat-input-abc.json",
        json!({"turns": turns, "language": "English"})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let pending = history(&root, "chat-abc.json");
    assert_eq!(pending.len(), 1, "an unanswered question is kept");
    store(
        &root,
        "chat-abc.json",
        json!({"answer": "A ladder game."}).to_string().as_bytes(),
    )
    .unwrap();
    let answered = history(&root, "chat-abc.json");
    assert_eq!(answered.len(), 2);
    assert_eq!(answered[1].role, "assistant");
    assert_eq!(answered[1].text, "A ladder game.");
    assert!(
        history(&root, "../escape.json").is_empty(),
        "names are not paths"
    );
    std::fs::remove_dir_all(root).unwrap();
}
