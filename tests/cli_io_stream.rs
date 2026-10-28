use agentconductor::cli_io::{self, Rules};

fn event(seq: u64, kind: &str, data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "schema":"acio.event/1",
        "seq":seq,
        "ts":format!("2026-10-28T01:00:{:02}Z", seq - 1),
        "session_id":"s",
        "type":kind,
        "data":data
    })
}

fn stream(events: &[serde_json::Value]) -> String {
    events
        .iter()
        .map(serde_json::Value::to_string)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn valid_session_and_result_events_pass_stream_validation() {
    let input = stream(&[
        event(1, "session", serde_json::json!({"session_id":"s"})),
        event(
            2,
            "result",
            serde_json::json!({"status":"success","summary":"done"}),
        ),
    ]);

    assert_eq!(
        cli_io::validate_stream("events", &input, Rules::default()),
        Ok(())
    );
}

#[test]
fn stream_validation_rejects_sequence_gaps_and_events_after_result() {
    let gap = stream(&[
        event(1, "session", serde_json::json!({"session_id":"s"})),
        event(
            3,
            "result",
            serde_json::json!({"status":"success","summary":"done"}),
        ),
    ]);
    assert!(
        cli_io::validate_stream("events", &gap, Rules::default())
            .unwrap_err()
            .contains("seq")
    );

    let after_result = stream(&[
        event(1, "session", serde_json::json!({"session_id":"s"})),
        event(
            2,
            "result",
            serde_json::json!({"status":"success","summary":"done"}),
        ),
        event(3, "heartbeat", serde_json::json!({})),
    ]);
    assert!(
        cli_io::validate_stream("events", &after_result, Rules::default())
            .unwrap_err()
            .contains("after")
    );
}
