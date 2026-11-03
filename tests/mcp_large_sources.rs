#![cfg(feature = "test-fixtures")]
use agentconductor::mcp::{self, Settings};
use serde_json::json;
#[tokio::test]
async fn large_unicode_source_is_preserved_beyond_transport_and_context_limits() {
    let root = std::env::temp_dir().join(format!(
        "ac-large-source-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let settings = Settings::parse(&json!({"mcp":{"docs":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","large_sources",root.join("trace")] }},"goal_sources":{"jira":{"server":"docs","tool":"jira_get_issue","arguments":{"issue_key":"{id}"},"read_only":true}}}).to_string()).unwrap();
    let goal = "jira:AC-1";
    let preview = mcp::preview(&settings, goal, &root).await.unwrap();
    assert!(preview.failures.is_empty(), "{:?}", preview.failures);
    assert!(preview.records[0].body.len() > 256 * 1024);
    assert!(preview.records[0].body.contains("TAIL_REQUIREMENT"));
    let applied = mcp::apply(&root, &preview, &settings, goal, &["jira:AC-1".into()]).unwrap();
    let data = mcp::load(
        &root
            .join("mcp")
            .join(format!("applied-{}.json", applied.id)),
    )
    .unwrap();
    let reloaded: mcp::Preview = serde_json::from_slice(&data).unwrap();
    assert_eq!(reloaded.records[0].sha256, preview.records[0].sha256);
    std::fs::remove_dir_all(root).unwrap();
}
