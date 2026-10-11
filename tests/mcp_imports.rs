use agentconductor::mcp::{self, Settings};
use serde_json::json;
use std::path::PathBuf;
fn root() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ac-mcp-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&p).unwrap();
    p.canonicalize().unwrap()
}
fn settings(mode: &str, root: &std::path::Path) -> Settings {
    let python = if cfg!(windows) { "python" } else { "python3" };
    Settings::parse(&json!({"mcp":{"company":{"type":"local","command":[python,concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_server.py"),mode,root.join("trace.jsonl")],"timeout":500}},"goal_sources":{"jira":{"server":"company","tool":"get_issue","arguments":{"id":"{id}"},"read_only":true},"confluence":{"server":"company","tool":"get_issue","arguments":{"id":"{id}"},"read_only":true}}}).to_string()).unwrap()
}
#[tokio::test]
async fn preview_is_read_only_partial_and_approval_is_goal_and_content_bound() {
    let root = root();
    let s = settings("normal", &root);
    let goal = "Implement jira:EMM-52599 with confluence:123 and confluence:999";
    let p = mcp::preview(&s, goal, &root).await.unwrap();
    assert_eq!(p.records.len(), 2);
    assert_eq!(p.failures.len(), 1);
    assert!(!p.failures[0].error.contains("secret"));
    assert!(mcp::apply(&root, &p, &s, goal, &["confluence:123".into()]).is_err());
    let selected = vec!["jira:EMM-52599".into(), "confluence:123".into()];
    let a = mcp::apply(&root, &p, &s, goal, &selected).unwrap();
    let text = mcp::context(&root, &a, &s, goal).unwrap();
    assert!(text.contains("Replay is rejected"));
    assert!(text.contains("SHA-256"));
    assert!(!text.contains("confluence:999"));
    assert!(mcp::context(&root, &a, &s, "other goal").is_err());
    let mut tampered = p.clone();
    tampered.records[0].body.push_str("changed");
    assert!(mcp::apply(&root, &tampered, &s, goal, &selected).is_err());
    std::fs::write(
        root.join("mcp").join(format!("applied-{}.json", a.id)),
        "changed",
    )
    .unwrap();
    assert!(mcp::context(&root, &a, &s, goal).is_err());
    let trace = std::fs::read_to_string(root.join("trace.jsonl")).unwrap();
    assert!(trace.contains("notifications/initialized"));
    assert_eq!(
        trace
            .lines()
            .filter(|s| s.contains("\"method\": \"initialize\""))
            .count(),
        1,
        "one local MCP process/session per server for the entire import"
    );
    assert!(!trace.contains("write_issue"));
    std::fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn malicious_tool_annotations_malformed_stream_and_timeout_are_refused() {
    for mode in [
        "write",
        "malformed",
        "hang",
        "paging",
        "schema",
        "oversized",
    ] {
        let root = root();
        let s = settings(mode, &root);
        let started = std::time::Instant::now();
        let p = mcp::preview(&s, "jira:EMM-1", &root).await.unwrap();
        assert_eq!(p.records.len(), 0);
        assert_eq!(p.failures.len(), 1);
        assert!(started.elapsed() < std::time::Duration::from_secs(3));
        if mode == "hang" {
            tokio::time::sleep(std::time::Duration::from_millis(700)).await;
            assert!(
                !root.join("trace.jsonl.leak").exists(),
                "timed-out MCP descendant escaped cleanup"
            );
        }
        if ["write", "paging", "schema"].contains(&mode) {
            assert!(
                !std::fs::read_to_string(root.join("trace.jsonl"))
                    .unwrap()
                    .contains("tools/call")
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
