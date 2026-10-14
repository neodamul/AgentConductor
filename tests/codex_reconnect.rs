use agentconductor::{
    coordinator,
    protocol::{Decoder, EventKind, Provider},
};
fn stream(final_event: &str) -> String {
    let mut text =
        String::from("{\"type\":\"thread.started\",\"thread_id\":\"enterprise-session\"}\n");
    for n in 1..=5 {
        text.push_str(
            &serde_json::json!({"type":"error","message":format!("Reconnecting... {n}/5")})
                .to_string(),
        );
        text.push('\n');
    }
    text.push_str(
        "{\"type\":\"error\",\"message\":\"Falling back from WebSockets to HTTPS transport.\"}\n",
    );
    text.push_str("{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"{\\\"tasks\\\":[]}\"}}\n");
    text.push_str(final_event);
    text.push('\n');
    text
}
#[test]
fn enterprise_reconnect_attempts_and_http_fallback_are_visible_nonterminal_warnings() {
    let mut decoder = Decoder::new(Provider::Codex, 4096);
    let events = decoder
        .push(stream("{\"type\":\"turn.completed\"}").as_bytes())
        .unwrap();
    assert!(events[1..7].iter().all(|e| e.kind == EventKind::Warning));
    assert_eq!(events.last().unwrap().kind, EventKind::Result);
    assert_eq!(events[1].session_id.as_deref(), Some("enterprise-session"));
    assert_eq!(events[1].payload["message"], "Reconnecting... 1/5");
    for message in [
        "stream disconnected before completion",
        "Reconnecting... failed",
        "Reconnecting... 0/5",
        "Reconnecting... 6/5",
        "unauthorized HTTP 401",
        "HTTP 429 quota exceeded",
    ] {
        let line = serde_json::json!({"type":"error","message":message}).to_string() + "\n";
        assert_eq!(
            Decoder::new(Provider::Codex, 4096)
                .push(line.as_bytes())
                .unwrap()[0]
                .kind,
            EventKind::Error,
            "{message}"
        );
    }
}
#[test]
fn coordinator_waits_for_final_turn_after_reconnect_and_still_refuses_terminal_failure() {
    let root = std::env::temp_dir().join(format!(
        "ac-codex-reconnect-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let log = root.join("stdout.jsonl");
    std::fs::write(&log, stream("{\"type\":\"turn.completed\"}")).unwrap();
    assert_eq!(
        coordinator::read_document(Provider::Codex, &log, "tasks").unwrap()["tasks"],
        serde_json::json!([])
    );
    for final_event in [
        "{\"type\":\"turn.failed\",\"error\":{\"message\":\"Reconnecting... 5/5\"}}",
        "{\"type\":\"error\",\"message\":\"stream disconnected before completion\"}",
        "",
    ] {
        std::fs::write(&log, stream(final_event)).unwrap();
        assert!(coordinator::read_document(Provider::Codex, &log, "tasks").is_err());
    }
    std::fs::remove_dir_all(root).unwrap();
}
