//! GitHub #100 (DD-169): provider-neutral current-tool extraction from normalized events.
use agentconductor::activity::{Change, tool_change};
use serde_json::json;

#[test]
fn claude_tool_use_starts_and_its_result_ends_the_tool() {
    let start = json!({"type":"assistant","message":{"content":[
        {"type":"text","text":"running"},
        {"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"cargo test"}}]}});
    assert_eq!(tool_change(&start), Change::Start("Bash".into()));
    let end = json!({"type":"user","message":{"content":[
        {"type":"tool_result","tool_use_id":"toolu_1","content":"ok"}]}});
    assert_eq!(tool_change(&end), Change::End);
    let text = json!({"type":"assistant","message":{"content":[{"type":"text","text":"hi"}]}});
    assert_eq!(tool_change(&text), Change::None);
}

#[test]
fn codex_items_start_and_complete_a_tool() {
    let start = json!({"type":"item.started","item":{"id":"i1","type":"command_execution","command":"ls -la"}});
    assert_eq!(tool_change(&start), Change::Start("ls -la".into()));
    let mcp = json!({"type":"item.started","item":{"id":"i2","type":"mcp_tool_call","server":"jira","tool":"get_issue"}});
    assert_eq!(tool_change(&mcp), Change::Start("jira.get_issue".into()));
    let done = json!({"type":"item.completed","item":{"id":"i1","type":"command_execution","command":"ls -la"}});
    assert_eq!(tool_change(&done), Change::End);
    let message =
        json!({"type":"item.completed","item":{"id":"i3","type":"agent_message","text":"done"}});
    assert_eq!(tool_change(&message), Change::None);
}

#[test]
fn custom_tool_events_start_a_named_tool_and_other_output_is_neutral() {
    assert_eq!(
        tool_change(&json!({"type":"tool","name":"formatter"})),
        Change::Start("formatter".into())
    );
    assert_eq!(tool_change(&json!({"type":"progress"})), Change::None);
    assert_eq!(tool_change(&json!({})), Change::None);
}

#[test]
fn long_tool_names_are_bounded() {
    let long = "x".repeat(500);
    let change = tool_change(
        &json!({"type":"item.started","item":{"type":"command_execution","command":long}}),
    );
    match change {
        Change::Start(name) => assert!(name.chars().count() <= 120),
        other => panic!("{other:?}"),
    }
}

#[test]
fn summary_reports_ages_the_tool_and_the_process_without_a_cause() {
    use agentconductor::activity::summarize;
    let doc = json!({"seq":9,"kind":"progress","observed_at_ms":1_000_000u64,
        "tool":{"name":"Bash","since_ms":940_000u64}});
    let s = summarize(Some(&doc), "present", 1_180_000);
    assert_eq!(s["quiet_ms"], 180_000);
    assert_eq!(s["tool"]["name"], "Bash");
    assert_eq!(s["tool"]["running_ms"], 240_000);
    assert_eq!(s["process"], "present");
    assert_eq!(s["kind"], "progress");
    let none = summarize(None, "absent", 5);
    assert!(none["quiet_ms"].is_null() && none["tool"].is_null());
    assert_eq!(none["process"], "absent");
    // A clock that went backwards never yields a negative or huge age.
    assert!(summarize(Some(&doc), "present", 10)["quiet_ms"].is_null());
}

