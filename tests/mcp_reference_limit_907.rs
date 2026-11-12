//! #907: a Goal holds at most five distinct Jira/Confluence references.
use agentconductor::mcp;
#[test]
fn five_distinct_references_pass_and_a_sixth_is_rejected_with_the_limit() {
    let five = "[Jira:A-1] [Jira:A-2] [Confluence:3] [Confluence:4] [Jira:A-5]";
    assert_eq!(mcp::references(five).unwrap().len(), 5);
    // Repeating a reference does not count twice.
    assert_eq!(
        mcp::references(&format!("{five} [Jira:A-1] [Confluence:3]"))
            .unwrap()
            .len(),
        5
    );
    let error = mcp::references(&format!("{five} [Jira:A-6]")).unwrap_err();
    assert!(error.contains('5') && error.contains('6'), "{error}");
}
