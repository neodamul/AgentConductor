use agentconductor::ui::Room;
use std::path::Path;

fn startup() -> (Room, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "ac-source-suggestions-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let room = Room::startup(&root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    (room, root)
}

fn suggestions(room: &Room, goal: &str) -> Vec<String> {
    analysis(room, goal)["suggestions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect()
}

fn analysis(room: &Room, goal: &str) -> serde_json::Value {
    let request = serde_json::json!({"kind":"mcp_source_suggestions","goal":goal}).to_string();
    room.dispatch(request.as_bytes()).unwrap()
}

#[test]
fn source_suggestions_cover_jira_confluence_explicit_refs_and_limits() {
    let (room, root) = startup();
    let document = room.document().unwrap();
    assert!(document.contains("kind:'mcp_source_suggestions'"));
    assert!(!document.contains("function sourceSuggestions"));
    assert_eq!(
        suggestions(
            &room,
            "Implement EMM-52599. 참고 (53553565, 548443335); refs: (53553565)"
        ),
        [
            "jira:EMM-52599",
            "confluence:53553565",
            "confluence:548443335"
        ]
    );
    assert!(suggestions(&room, "jira:EMM-52599 confluence:53553565").is_empty());
    assert!(suggestions(&room, "2026 build 500, prefixEMM-1").is_empty());
    let references = analysis(
        &room,
        "작업 jira:EMM-52599 and confluence:123 duplicate jira:EMM-52599",
    )["references"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(references, ["jira:EMM-52599", "confluence:123"]);
    assert_eq!(
        analysis(&room, "작업 jira:EMM-52599 and confluence:123")["reference_ranges"],
        serde_json::json!([
            {"reference":"jira:EMM-52599","start":3,"end":17},
            {"reference":"confluence:123","start":22,"end":36}
        ])
    );
    assert!(
        suggestions(
            &room,
            "references (53553565, 123456789012345678901234567890123)"
        )
        .is_empty()
    );

    let goal = (1..=15)
        .map(|n| format!("ABC-{n}"))
        .collect::<Vec<_>>()
        .join(" ");
    let values = suggestions(&room, &goal);
    assert_eq!(values.len(), 12);
    assert_eq!(values[0], "jira:ABC-1");
    assert_eq!(values[11], "jira:ABC-12");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
