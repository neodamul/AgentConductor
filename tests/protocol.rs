use agentconductor::protocol::{Decoder, EventKind, Provider};

#[test]
fn preserves_split_utf8_crlf_and_final_unterminated_line() {
    let bytes = "{\"type\":\"text\",\"sessionID\":\"oc-1\",\"part\":{\"text\":\"🦀\"}}\r\n{\"type\":\"step_finish\"}".as_bytes();
    let mut decoder = Decoder::new(Provider::OpenCode, 1024);
    let mut events = vec![];
    for byte in bytes {
        events.extend(decoder.push(&[*byte]).unwrap());
    }
    events.extend(decoder.finish().unwrap());
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].session_id.as_deref(), Some("oc-1"));
    assert_eq!(events[0].payload["part"]["text"], "🦀");
    assert_eq!(events[1].seq, 1);
    assert_eq!(events[1].kind, EventKind::Progress);
    assert_eq!(events[1].session_id.as_deref(), Some("oc-1"));
}

#[test]
fn claude_reports_init_and_error_result() {
    let mut d = Decoder::new(Provider::Claude, 1024);
    let events = d.push(b"{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"c-1\"}\n{\"type\":\"result\",\"is_error\":true,\"result\":\"denied\"}\n").unwrap();
    assert_eq!(events[0].kind, EventKind::Session);
    assert_eq!(events[1].kind, EventKind::Error);
    assert_eq!(events[1].session_id.as_deref(), Some("c-1"));
}

#[test]
fn codex_nonterminal_error_items_are_visible_warnings_and_do_not_replace_terminal_outcomes() {
    let mut decoder = Decoder::new(Provider::Codex, 1024);
    let events=decoder.push(b"{\"type\":\"item.completed\",\"item\":{\"type\":\"error\",\"message\":\"MCP startup warning\"}}\n{\"type\":\"turn.completed\"}\n").unwrap();
    assert_eq!(events[0].kind, EventKind::Warning);
    assert_eq!(events[1].kind, EventKind::Result);
}

#[test]
fn codex_message_is_not_completion_and_terminal_turn_has_usage() {
    let mut d = Decoder::new(Provider::Codex, 2048);
    let events = d.push(b"{\"type\":\"thread.started\",\"thread_id\":\"cx-1\"}\n{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"done\"}}\n{\"type\":\"turn.completed\",\"usage\":{\"input_tokens\":12}}\n").unwrap();
    assert_eq!(events[0].kind, EventKind::Session);
    assert_eq!(events[1].kind, EventKind::Progress);
    assert_eq!(events[2].kind, EventKind::Result);
    assert_eq!(events[2].payload["usage"]["input_tokens"], 12);
}

#[test]
fn unknown_events_are_preserved_with_raw_offsets() {
    let mut d = Decoder::new(Provider::Custom, 1024);
    let line = b"{\"type\":\"session\",\"session_id\":\"internal-1\"}\n";
    let first = d.push(line).unwrap();
    let second = d
        .push(b"{\"type\":\"vendor.new\",\"future\":42}\n")
        .unwrap();
    assert_eq!(first[0].raw_offset, 0);
    assert_eq!(second[0].raw_offset, line.len() as u64);
    assert_eq!(second[0].kind, EventKind::Unknown);
    assert_eq!(second[0].payload["future"], 42);
}

#[test]
fn malformed_json_invalid_utf8_and_oversized_lines_fail() {
    for bytes in [b"oops\n".as_slice(), b"[]\n", b"\xff\n"] {
        assert!(Decoder::new(Provider::Custom, 1024).push(bytes).is_err());
    }
    let mut d = Decoder::new(Provider::Custom, 4);
    assert!(d.push(b"12345").is_err());
}

#[test]
fn provider_failures_are_errors_and_opencode_step_is_not_result() {
    for (provider, line) in [
        (Provider::Codex, b"{\"type\":\"turn.failed\"}\n".as_slice()),
        (Provider::OpenCode, b"{\"type\":\"error\"}\n"),
        (Provider::Custom, b"{\"type\":\"error\"}\n"),
    ] {
        assert_eq!(
            Decoder::new(provider, 1024).push(line).unwrap()[0].kind,
            EventKind::Error
        );
    }
    assert_eq!(
        Decoder::new(Provider::OpenCode, 1024)
            .push(b"{\"type\":\"step_finish\"}\n")
            .unwrap()[0]
            .kind,
        EventKind::Progress
    );
}
