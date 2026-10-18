#![cfg(feature = "test-fixtures")]
use agentconductor::{
    mcp::{self, Settings},
    plan::AgentProfile,
};
use serde_json::json;
fn profile(mode: &str) -> AgentProfile {
    serde_json::from_value(json!({"id":"coordinator","executable":env!("CARGO_BIN_EXE_test_agent_fixture"),"args":["mcp-coordinator",mode],"parser":"custom","input_mode":"stdin","permission_mode":"read_only"})).unwrap()
}
#[tokio::test]
async fn coordinator_selects_across_servers_without_manual_bindings_and_summarizes_saved_sources() {
    let root = std::env::temp_dir().join(format!(
        "ac-auto-source-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let mut settings=Settings::parse(&json!({"mcp":{"jira-server":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","large_sources",root.join("jira.trace")]},"wiki-server":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","sources",root.join("wiki.trace")]}}}).to_string()).unwrap();
    let goal = "jira:AC-1 confluence:123";
    let p = mcp::preview_with_coordinator(&settings, goal, &root, &profile("normal"))
        .await
        .unwrap();
    assert!(p.failures.is_empty(), "{:?}", p.failures);
    assert_eq!(p.records.len(), 2);
    let a = mcp::apply(
        &root,
        &p,
        &settings,
        goal,
        &["jira:AC-1".into(), "confluence:123".into()],
    )
    .unwrap();
    let context = mcp::context(&root, &a, &settings, goal).unwrap();
    assert!(context.len() <= mcp::MAX_CONTEXT);
    assert!(context.contains("TAIL_REQUIREMENT"));
    assert!(context.contains("source-"));
    let evidence = &p.summaries["jira:AC-1"];
    let md = root.join("mcp").join(&evidence.markdown_file);
    let source = std::fs::read_to_string(&md).unwrap();
    assert!(source.contains("TAIL_REQUIREMENT"));
    assert!(source.len() > 256 * 1024);
    std::fs::write(&md, "tampered").unwrap();
    assert!(mcp::context(&root, &a, &settings, goal).is_err());
    settings.mcp.clear();
    assert!(mcp::context(&root, &a, &settings, goal).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn coordinator_cannot_call_a_write_or_undiscovered_tool() {
    let root = std::env::temp_dir().join(format!(
        "ac-auto-write-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let settings=Settings::parse(&json!({"mcp":{"docs":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","sources",root.join("trace")]}}}).to_string()).unwrap();
    let p = mcp::preview_with_coordinator(&settings, "jira:AC-1", &root, &profile("write"))
        .await
        .unwrap();
    assert!(p.records.is_empty());
    assert_eq!(p.failures.len(), 1);
    assert!(
        !std::fs::read_to_string(root.join("trace"))
            .unwrap()
            .contains("tools/call")
    );
    let unrelated =
        mcp::preview_with_coordinator(&settings, "jira:AC-1", &root, &profile("unrelated"))
            .await
            .unwrap();
    assert!(unrelated.records.is_empty());
    assert_eq!(unrelated.failures.len(), 1);
    assert!(
        !std::fs::read_to_string(root.join("trace"))
            .unwrap()
            .contains("tools/call")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn coordinator_reads_required_cloud_metadata_before_fetching_the_reference() {
    let root = std::env::temp_dir().join(format!(
        "ac-cloud-source-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let settings=Settings::parse(&json!({"mcp":{"docs":{"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","cloud_sources",root.join("trace")]}}}).to_string()).unwrap();
    let p = mcp::preview_with_coordinator(&settings, "jira:AC-1", &root, &profile("normal"))
        .await
        .unwrap();
    assert!(p.failures.is_empty(), "{:?}", p.failures);
    assert_eq!(p.records.len(), 1);
    let calls = std::fs::read_to_string(root.join("trace"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter(|v| v["method"] == "tools/call")
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[0]["params"]["name"],
        "getAccessibleAtlassianResources"
    );
    assert_eq!(calls[1]["params"]["arguments"]["cloudId"], "tenant-1");
    assert_eq!(calls[1]["params"]["arguments"]["issue_key"], "AC-1");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn large_multi_server_catalogs_are_narrowed_without_exceeding_coordinator_prompts() {
    let root = std::env::temp_dir().join(format!(
        "ac-many-tools-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let server = json!({"type":"local","command":[env!("CARGO_BIN_EXE_test_agent_fixture"),"mcp-server","many_tools",root.join("trace")]});
    let settings = Settings::parse(&json!({"mcp":{"a":server,"b":server}}).to_string()).unwrap();
    let p = mcp::preview_with_coordinator(&settings, "jira:AC-1", &root, &profile("normal"))
        .await
        .unwrap();
    assert!(p.failures.is_empty(), "{:?}", p.failures);
    assert_eq!(p.records.len(), 1);
    let mut prompts = vec![];
    for item in std::fs::read_dir(root.join("mcp")).unwrap() {
        let path = item.unwrap().path().join("prompt.txt");
        if path.exists() {
            let text = std::fs::read_to_string(path).unwrap();
            assert!(text.len() < 128 * 1024);
            prompts.push(text);
        }
    }
    assert!(prompts.iter().any(|text| text.contains("tools_count")));
    assert!(prompts.len() >= 2);
    std::fs::remove_dir_all(root).unwrap();
}
