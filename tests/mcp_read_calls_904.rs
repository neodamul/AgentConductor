//! #904: a successful read call is reused without the coordinator; a changed schema or a failing call falls back.
use agentconductor::{
    mcp::{self, Progress, Settings},
    plan::AgentProfile,
};
use serde_json::{Value, json};
fn profile(mode: &str) -> AgentProfile {
    serde_json::from_value(json!({"id":"coordinator","executable":if cfg!(windows){"python"}else{"python3"},"args":[concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_coordinator.py"),mode],"parser":"custom","input_mode":"stdin","permission_mode":"read_only"})).unwrap()
}
fn events(p: &Progress, reference: &str) -> Vec<String> {
    p.documents
        .iter()
        .find(|d| d.reference == reference)
        .unwrap()
        .log
        .iter()
        .map(|e| e.event.clone())
        .collect()
}
#[tokio::test]
async fn stored_read_call_skips_selection_and_falls_back_on_change_or_error() {
    let root = std::env::temp_dir().join(format!(
        "ac-read-calls-904-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let python = if cfg!(windows) { "python" } else { "python3" };
    let settings=Settings::parse(&json!({"mcp":{"docs":{"type":"local","timeout":30000,"command":[python,concat!(env!("CARGO_MANIFEST_DIR"),"/tests/support/mcp_server.py"),"sources",root.join("trace")]}}}).to_string()).unwrap();
    let run = |goal: &'static str, mode: &'static str| {
        let (settings, root) = (settings.clone(), root.clone());
        async move {
            let mut progress = Progress::off();
            let p = mcp::preview_with_progress(
                &settings,
                goal,
                &root,
                &profile(mode),
                &mut progress,
                None,
            )
            .await
            .unwrap();
            (p, progress)
        }
    };
    let file = root.join("mcp").join("read-calls.json");
    let (first, _) = run("[Jira:AC-1] [Confluence:123]", "normal").await;
    assert!(first.failures.is_empty(), "{:?}", first.failures);
    let stored: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(stored["jira"]["tool"], "jira_get_issue");
    assert!(
        !stored.to_string().contains("AC-1"),
        "the reference ID is a placeholder"
    );
    // A coordinator that cannot select proves the stored calls were used for new references.
    let (reused, progress) = run("[Jira:AC-2] [Confluence:456]", "write").await;
    assert!(reused.failures.is_empty(), "{:?}", reused.failures);
    assert_eq!(reused.records.len(), 2);
    let log = events(&progress, "jira:AC-2");
    assert!(
        log.contains(&"stored_call".into()) && !log.contains(&"coordinator".into()),
        "{log:?}"
    );
    // A changed schema discards the stored call and asks the coordinator again, which stores it anew.
    let mut tampered = stored.clone();
    tampered["jira"]["schema_sha256"] = json!("changed");
    std::fs::write(&file, tampered.to_string()).unwrap();
    let (again, progress) = run("[Jira:AC-3]", "normal").await;
    assert!(again.failures.is_empty(), "{:?}", again.failures);
    let log = events(&progress, "jira:AC-3");
    assert!(
        log.contains(&"stored_discarded".into()) && log.contains(&"coordinator".into()),
        "{log:?}"
    );
    let restored: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(restored["jira"], stored["jira"]);
    // A failing stored call is dropped and selection runs (the fixture rejects page 999 either way).
    let (failed, progress) = run("[Confluence:999]", "normal").await;
    assert_eq!(failed.failures.len(), 1);
    let log = events(&progress, "confluence:999");
    assert!(
        log.contains(&"stored_failed".into()) && log.contains(&"coordinator".into()),
        "{log:?}"
    );
    let left: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert!(left.get("confluence").is_none() && left.get("jira").is_some());
    std::fs::remove_dir_all(root).unwrap();
}
