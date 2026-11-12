//! #898: selection and ordinary summaries need no coordinator file-tool round trip.
use agentconductor::{
    mcp::{self, Settings},
    plan::AgentProfile,
};
use serde_json::json;
#[tokio::test]
async fn inspected_schema_and_bounded_source_are_supplied_inline() {
    for (server_mode, coordinator_mode) in [
        ("sources", "schema_inline"),
        ("inline_sources", "inline"),
        ("strict_issue", "strict_issue"),
        ("schema_file_sources", "schema_file"),
    ] {
        let root = std::env::temp_dir().join(format!(
            "ac-preview-898-{}",
            agentconductor::native_guard::session_id().unwrap()
        ));
        std::fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let python = if cfg!(windows) { "python" } else { "python3" };
        let settings=Settings::parse(&json!({"mcp":{"docs":{"type":"local","command":[python,concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_server.py"),server_mode,root.join("trace")]}}}).to_string()).unwrap();
        let profile:AgentProfile=serde_json::from_value(json!({"id":"coordinator","executable":python,"args":[concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_coordinator.py"),coordinator_mode],"parser":"custom","input_mode":"stdin","permission_mode":"read_only"})).unwrap();
        let preview = mcp::preview_with_coordinator(&settings, "[Jira:AC-1]", &root, &profile)
            .await
            .unwrap();
        assert!(preview.failures.is_empty(), "{:?}", preview.failures);
        assert_eq!(preview.records.len(), 1);
        let saved = &preview.summaries["jira:AC-1"];
        assert!(root.join("mcp").join(&saved.markdown_file).is_file());
        let receipt = mcp::apply(
            &root,
            &preview,
            &settings,
            "[Jira:AC-1]",
            &["jira:AC-1".into()],
        )
        .unwrap();
        assert!(
            mcp::context(&root, &receipt, &settings, "[Jira:AC-1]")
                .unwrap()
                .contains("Replay is rejected")
        );
        if server_mode == "strict_issue" {
            let calls: Vec<serde_json::Value> = std::fs::read_to_string(root.join("trace"))
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .filter(|v: &serde_json::Value| v["method"] == "tools/call")
                .collect();
            assert_eq!(
                calls.len(),
                1,
                "the guessed invalid arguments must never reach MCP"
            );
            assert!(
                calls[0]["params"]["arguments"]
                    .get("issueIdOrKey")
                    .is_none()
            );
            assert_eq!(calls[0]["params"]["arguments"]["issueKey"], "AC-1");
            assert!(calls[0]["params"]["arguments"]["fieldIds"].is_array());
            assert_eq!(calls[0]["params"]["arguments"]["expand"], "renderedFields");
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
