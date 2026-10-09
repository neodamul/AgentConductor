use agentconductor::{coordinator, protocol::Provider};

#[test]
fn review_proposals_cannot_invent_tasks_evidence_commands_or_merge_approval() {
    let valid = serde_json::json!({"verdict":"revise","summary":"Repair the failed task","evidence":["agent:one"],"revisions":[{"task":"one","feedback":"Fix the failing case"}]});
    let proposal: coordinator::ReviewProposal = serde_json::from_value(valid.clone()).unwrap();
    assert!(
        proposal
            .validate(&["one".into()], &["agent:one".into()], false)
            .is_ok()
    );
    let mut invalid = valid.clone();
    invalid["revisions"][0]["task"] = "invented".into();
    let proposal: coordinator::ReviewProposal = serde_json::from_value(invalid).unwrap();
    assert!(
        proposal
            .validate(&["one".into()], &["agent:one".into()], false)
            .is_err()
    );
    let mut invalid = valid.clone();
    invalid["evidence"][0] = "test:invented".into();
    let proposal: coordinator::ReviewProposal = serde_json::from_value(invalid).unwrap();
    assert!(
        proposal
            .validate(&["one".into()], &["agent:one".into()], false)
            .is_err()
    );
    let mut invalid = valid.clone();
    invalid["command"] = "deploy".into();
    assert!(serde_json::from_value::<coordinator::ReviewProposal>(invalid).is_err());
    let accept = serde_json::json!({"verdict":"accept","summary":"Looks good","evidence":["agent:one"],"revisions":[]});
    let proposal: coordinator::ReviewProposal = serde_json::from_value(accept).unwrap();
    assert!(
        proposal
            .validate(&["one".into()], &["agent:one".into()], false)
            .is_err()
    );
    assert!(
        proposal
            .validate(&["one".into()], &["agent:one".into()], true)
            .is_ok()
    );
}

fn spool(name: &str, text: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ac-review-json-{}-{name}.jsonl",
        std::process::id()
    ));
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn final_document_requires_typed_model_output_and_a_successful_terminal_event() {
    let path = spool(
        "terminal",
        "{\"type\":\"progress\",\"payload\":{\"revisions\":[]}}\n",
    );
    assert!(coordinator::read_document(Provider::Custom, &path, "revisions").is_err());
    std::fs::write(
        &path,
        "{\"type\":\"result\",\"payload\":{\"revisions\":[]}}\n",
    )
    .unwrap();
    assert_eq!(
        coordinator::read_document(Provider::Custom, &path, "revisions").unwrap()["revisions"],
        serde_json::json!([])
    );
    std::fs::write(&path,"{\"type\":\"result\",\"payload\":{\"revisions\":[]}}\n{\"type\":\"error\",\"error\":\"rejected\"}\n").unwrap();
    assert!(coordinator::read_document(Provider::Custom, &path, "revisions").is_err());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn codex_tool_output_is_not_a_coordinator_decision() {
    let path = spool(
        "tool",
        "{\"type\":\"item.completed\",\"item\":{\"type\":\"command_execution\",\"text\":\"{\\\"revisions\\\":[]}\"}}\n{\"type\":\"turn.completed\"}\n",
    );
    assert!(coordinator::read_document(Provider::Codex, &path, "revisions").is_err());
    std::fs::write(&path,"{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"{\\\"revisions\\\":[]}\"}}\n{\"type\":\"turn.completed\"}\n").unwrap();
    assert!(coordinator::read_document(Provider::Codex, &path, "revisions").is_ok());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn opencode_accumulated_output_is_bounded() {
    let line =
        serde_json::json!({"type":"text","part":{"text":"x".repeat(40*1024)}}).to_string() + "\n";
    let path = spool("bounded", &(line.repeat(2)));
    let error = coordinator::read_document(Provider::OpenCode, &path, "revisions").unwrap_err();
    assert!(error.contains("64 KiB"));
    std::fs::remove_file(path).unwrap();
}
