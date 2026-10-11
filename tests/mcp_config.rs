use agentconductor::mcp::{Settings, references};
use serde_json::json;
#[test]
fn opencode_jsonc_and_explicit_references() {
    let settings = Settings::parse(r#"{// comment
      "mcp":{"company":{"type":"local","command":["company-mcp"],"environment":{"TOKEN":"{env:COMPANY_TOKEN}"},"enabled":true,}},
      "goal_sources":{"jira":{"server":"company","tool":"get_issue","arguments":{"issue_key":"{id}"},"read_only":true}}
    }"#).unwrap();
    assert_eq!(settings.mcp.len(), 1);
    assert_eq!(
        references("Implement jira:EMM-52599. Ref confluence:53553565, confluence:53553565")
            .unwrap()
            .len(),
        2
    );
    assert!(references("EMM-52599 and 53553565").unwrap().is_empty());
    assert!(
        Settings::parse(&json!({"mcp":{"x":{"type":"local","command":[]}}}).to_string()).is_err()
    );
}
#[test]
fn credentials_must_be_references_and_tools_explicitly_read_only() {
    for value in [
        json!({"mcp":{"x":{"type":"remote","url":"https://example.org/mcp","headers":{"Authorization":"Bearer secret"}}}}),
        json!({"mcp":{"x":{"type":"local","command":["mcp"]}},"goal_sources":{"jira":{"server":"x","tool":"write_issue","arguments":{"id":"{id}"},"read_only":false}}}),
    ] {
        assert!(Settings::parse(&value.to_string()).is_err());
    }
}
