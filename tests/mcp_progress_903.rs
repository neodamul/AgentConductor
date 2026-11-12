//! #903: a document preview reports each document's step, and a retry keeps documents already read.
use agentconductor::{
    mcp::{self, Progress, Settings},
    plan::AgentProfile,
};
use serde_json::{Value, json};
fn profile(mode: &str) -> AgentProfile {
    serde_json::from_value(json!({"id":"coordinator","executable":if cfg!(windows){"python"}else{"python3"},"args":[concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_coordinator.py"),mode],"parser":"custom","input_mode":"stdin","permission_mode":"read_only"})).unwrap()
}
#[tokio::test]
async fn progress_file_tracks_every_document_and_retry_carries_read_documents() {
    let root = std::env::temp_dir().join(format!(
        "ac-progress-903-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let python = if cfg!(windows) { "python" } else { "python3" };
    let settings=Settings::parse(&json!({"mcp":{"docs":{"type":"local","timeout":30000,"command":[python,concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_server.py"),"sources",root.join("trace")]}}}).to_string()).unwrap();
    let goal = "[Jira:AC-1] [Confluence:123]";
    let path = root.join("mcp").join("progress-a.json");
    let first = mcp::preview_with_progress(
        &settings,
        goal,
        &root,
        &profile("normal"),
        &mut Progress::file(path.clone()),
        None,
    )
    .await
    .unwrap();
    assert!(first.failures.is_empty(), "{:?}", first.failures);
    let written: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let docs = written["documents"].as_array().unwrap();
    assert_eq!(docs.len(), 2);
    for doc in docs {
        assert_eq!(doc["stage"], "done", "{doc}");
        let events: Vec<_> = doc["log"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["event"].as_str().unwrap())
            .collect();
        for event in [
            "coordinator",
            "selected",
            "mcp_call",
            "read_done",
            "summary_done",
        ] {
            assert!(events.contains(&event), "{event} missing in {events:?}");
        }
        assert!(doc["started_ms"].as_u64().unwrap() <= doc["ended_ms"].as_u64().unwrap());
    }
    // A coordinator that cannot pick a read tool proves the retry fetched nothing again.
    let mut progress = Progress::off();
    let retried = mcp::preview_with_progress(
        &settings,
        goal,
        &root,
        &profile("write"),
        &mut progress,
        Some(first.clone()),
    )
    .await
    .unwrap();
    assert!(retried.failures.is_empty(), "{:?}", retried.failures);
    assert_eq!(retried.records.len(), 2);
    assert_eq!(retried.summaries.len(), first.summaries.len());
    assert!(
        progress
            .documents
            .iter()
            .all(|d| d.stage == "done" && d.log.iter().any(|e| e.event == "carried"))
    );
    // A changed goal does not reuse the earlier documents.
    let mut progress = Progress::off();
    let changed = mcp::preview_with_progress(
        &settings,
        "[Jira:AC-1]",
        &root,
        &profile("write"),
        &mut progress,
        Some(first),
    )
    .await
    .unwrap();
    assert!(changed.records.is_empty());
    assert_eq!(progress.documents[0].stage, "failed");
    std::fs::remove_dir_all(root).unwrap();
}
