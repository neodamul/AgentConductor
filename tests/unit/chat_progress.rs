use super::items;
use serde_json::json;

fn log(lines: &[serde_json::Value]) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ac-chat-progress-{}",
        std::process::id() as u64 * 1000 + lines.len() as u64
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    std::fs::write(dir.join("stdout.jsonl"), text).unwrap();
    dir
}

#[test]
fn claude_and_codex_events_become_short_tool_and_text_items() {
    let dir = log(&[
        json!({"type":"system","subtype":"init"}),
        json!({"type":"assistant","message":{"content":[
            {"type":"text","text":"Reading the project\nsecond line"},
            {"type":"tool_use","id":"a","name":"Read","input":{"file_path":"src/main.rs"}}]}}),
        json!({"type":"item.started","item":{"id":"i1","type":"command_execution","command":"ls -la"}}),
        json!({"type":"item.started","item":{"id":"i2","type":"mcp_tool_call","server":"jira","tool":"get_issue"}}),
        json!({"type":"item.completed","item":{"id":"i3","type":"agent_message","text":"done"}}),
    ]);
    let got = items(&dir);
    let texts: Vec<_> = got.iter().map(|i| i["text"].as_str().unwrap()).collect();
    assert_eq!(
        texts,
        [
            "Reading the project",
            "Read src/main.rs",
            "command ls -la",
            "jira.get_issue",
            "done"
        ]
    );
    assert_eq!(got[1]["kind"], "tool");
    assert_eq!(got[0]["kind"], "text");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn items_are_bounded_and_garbage_is_skipped() {
    let mut lines = vec![];
    for n in 0..20 {
        lines.push(json!({"type":"assistant","message":{"content":[{"type":"tool_use","name":format!("Tool{n}"),"input":{"command":"x".repeat(500)}}]}}));
    }
    let dir = log(&lines);
    let mut raw = std::fs::read_to_string(dir.join("stdout.jsonl")).unwrap();
    raw.push_str("not json\n{\"type\":\n");
    std::fs::write(dir.join("stdout.jsonl"), raw).unwrap();
    let got = items(&dir);
    assert_eq!(got.len(), 8, "at most eight items");
    assert!(got[7]["text"].as_str().unwrap().starts_with("Tool19"));
    assert!(
        got.iter()
            .all(|i| i["text"].as_str().unwrap().chars().count() <= 121)
    );
    assert!(items(&dir.join("missing")).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}
