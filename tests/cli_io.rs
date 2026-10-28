//! GitHub #207: the acio schemas ship in the binary, a small checker validates documents against them, and the
//! stream rules JSON Schema cannot express are checked on a whole captured stream.
use agentconductor::cli_io::{self, Rules};
use std::{fs, path::Path};

const DIR: &str = "schemas/cli-io/v1";

fn kind_of(file: &str) -> &'static str {
    match file {
        f if f.starts_with("agent") => "agent",
        f if f.starts_with("describe") => "describe",
        f if f.starts_with("input") => "input",
        f if f.starts_with("control") => "control",
        _ => "event",
    }
}
fn documents(path: &Path) -> Vec<serde_json::Value> {
    let text = fs::read_to_string(path).unwrap();
    if path.extension().is_some_and(|e| e == "jsonl") {
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    } else {
        vec![serde_json::from_str(&text).unwrap()]
    }
}
fn files(sub: &str) -> Vec<std::path::PathBuf> {
    let mut v: Vec<_> = fs::read_dir(format!("{DIR}/{sub}"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    v.sort();
    v
}

#[test]
fn every_embedded_schema_matches_the_file_and_parses() {
    for kind in cli_io::KINDS {
        let text = cli_io::schema(kind).unwrap();
        assert_eq!(
            text,
            fs::read_to_string(format!("{DIR}/{kind}.schema.json")).unwrap()
        );
        assert!(
            serde_json::from_str::<serde_json::Value>(text).unwrap()["$id"]
                .as_str()
                .unwrap()
                .contains(kind)
        );
    }
    assert!(cli_io::schema("nope").is_err());
}

#[test]
fn valid_examples_pass_and_invalid_examples_are_refused() {
    for path in files("examples/valid")
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "json" || e == "jsonl"))
    {
        let kind = kind_of(path.file_name().unwrap().to_str().unwrap());
        for doc in documents(&path) {
            assert_eq!(cli_io::validate(kind, &doc), Ok(()), "{path:?}");
        }
    }
    for path in files("examples/invalid") {
        let kind = kind_of(path.file_name().unwrap().to_str().unwrap());
        let refused = documents(&path)
            .iter()
            .any(|doc| cli_io::validate(kind, doc).is_err());
        assert!(refused, "{path:?} should be refused");
    }
}

#[test]
fn shipped_presets_are_valid_agent_settings() {
    for name in ["claude", "codex"] {
        let doc = serde_json::from_str(
            &fs::read_to_string(format!("{DIR}/presets/{name}.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(cli_io::validate("agent", &doc), Ok(()), "{name}");
    }
}

#[test]
fn a_refusal_names_the_place_and_the_reason() {
    let doc = serde_json::json!({"schema":"acio.event/1","seq":0,"ts":"2026-10-28T01:00:00Z","type":"progress","data":{"message":"x"}});
    let errors = cli_io::validate("event", &doc).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("/seq:") && e.contains("minimum")),
        "{errors:?}"
    );
    let doc = serde_json::json!({"schema":"acio.event/1","seq":1,"ts":"yesterday","type":"progress","data":{"message":"x"}});
    assert!(
        cli_io::validate("event", &doc)
            .unwrap_err()
            .iter()
            .any(|e| e.starts_with("/ts:"))
    );
}

fn stream(lines: &[serde_json::Value]) -> String {
    lines
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
fn ev(seq: u64, ts: &str, kind: &str, data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({"schema":"acio.event/1","seq":seq,"ts":ts,"type":kind,"session_id":"s","data":data})
}

#[test]
fn stream_rules_catch_what_a_single_line_cannot_show() {
    let session = |s| {
        ev(
            s,
            "2026-10-28T01:00:00Z",
            "session",
            serde_json::json!({"session_id":"s"}),
        )
    };
    let result = |s, t: &str| {
        ev(
            s,
            t,
            "result",
            serde_json::json!({"status":"success","summary":"ok","data":{"a":1}}),
        )
    };
    let ok = stream(&[session(1), result(2, "2026-10-28T01:00:01Z")]);
    assert_eq!(
        cli_io::validate_stream("events", &ok, Rules::default()),
        Ok(())
    );
    let cases = [
        (
            "a gap in seq",
            stream(&[session(1), result(3, "2026-10-28T01:00:01Z")]),
            "seq",
        ),
        (
            "time going backwards",
            stream(&[session(1), result(2, "2026-10-28T00:59:00Z")]),
            "ts",
        ),
        ("no result", stream(&[session(1)]), "result"),
        (
            "an event after the result",
            stream(&[
                session(1),
                result(2, "2026-10-28T01:00:01Z"),
                ev(
                    3,
                    "2026-10-28T01:00:02Z",
                    "heartbeat",
                    serde_json::json!({}),
                ),
            ]),
            "after",
        ),
        (
            "two results",
            stream(&[
                session(1),
                result(2, "2026-10-28T01:00:01Z"),
                result(3, "2026-10-28T01:00:02Z"),
            ]),
            "result",
        ),
        (
            "a session id without a session event",
            stream(&[
                ev(
                    1,
                    "2026-10-28T01:00:00Z",
                    "progress",
                    serde_json::json!({"message":"m"}),
                ),
                result(2, "2026-10-28T01:00:01Z"),
            ]),
            "session",
        ),
        (
            "an invalid line",
            "{\"schema\":\"acio.event/1\"}\n".to_string(),
            "line 1",
        ),
        ("text that is not JSON", "hello\n".to_string(), "line 1"),
    ];
    for (name, text, needle) in cases {
        let err = cli_io::validate_stream("events", &text, Rules::default()).unwrap_err();
        assert!(err.contains(needle), "{name}: {err}");
    }
}

#[test]
fn a_coordinator_result_must_carry_typed_data() {
    let bare = stream(&[
        ev(
            1,
            "2026-10-28T01:00:00Z",
            "session",
            serde_json::json!({"session_id":"s"}),
        ),
        ev(
            2,
            "2026-10-28T01:00:01Z",
            "result",
            serde_json::json!({"status":"success","summary":"ok"}),
        ),
    ]);
    assert_eq!(
        cli_io::validate_stream("events", &bare, Rules::default()),
        Ok(())
    );
    let err = cli_io::validate_stream("events", &bare, Rules { coordinator: true }).unwrap_err();
    assert!(err.contains("data"), "{err}");
}

#[test]
fn control_streams_count_from_one() {
    let line = |seq, t: &str, d: serde_json::Value| serde_json::json!({"schema":"acio.control/1","seq":seq,"type":t,"data":d});
    let ok = stream(&[
        line(1, "user_message", serde_json::json!({"text":"hi"})),
        line(2, "cancel", serde_json::json!({})),
    ]);
    assert_eq!(
        cli_io::validate_stream("controls", &ok, Rules::default()),
        Ok(())
    );
    let bad = stream(&[line(2, "cancel", serde_json::json!({}))]);
    assert!(
        cli_io::validate_stream("controls", &bad, Rules::default())
            .unwrap_err()
            .contains("seq")
    );
}

#[cfg(unix)]
#[test]
fn the_minimal_cli_example_speaks_the_protocol() {
    let script = format!("{DIR}/examples/minimal-cli.sh");
    let run = |args: &[&str], stdin: &str| {
        use std::io::Write;
        let mut child = std::process::Command::new("sh")
            .arg(&script)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
    };
    let describe: serde_json::Value = serde_json::from_str(&run(&["--acio-describe"], "")).unwrap();
    assert_eq!(cli_io::validate("describe", &describe), Ok(()));
    let input = fs::read_to_string(format!("{DIR}/examples/valid/input-ask.json")).unwrap();
    assert_eq!(
        cli_io::validate_stream("events", &run(&[], &input), Rules::default()),
        Ok(())
    );
}

#[test]
fn the_io_command_prints_schemas_and_validates_files() {
    let exe = env!("CARGO_BIN_EXE_agentconductor");
    let out = std::process::Command::new(exe)
        .args(["io", "schema", "event"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        cli_io::schema("event").unwrap()
    );
    let ok = format!("{DIR}/examples/valid/events-chat.jsonl");
    let out = std::process::Command::new(exe)
        .args(["io", "validate", "events", &ok])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let bad = format!("{DIR}/examples/invalid/event-seq-zero.jsonl");
    let out = std::process::Command::new(exe)
        .args(["io", "validate", "events", &bad])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("line 1"));
    let out = std::process::Command::new(exe)
        .args([
            "io",
            "validate",
            "agent",
            &format!("{DIR}/presets/codex.json"),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
}

#[test]
fn the_pattern_subset_and_time_zones_behave() {
    let input = |language: &str| serde_json::json!({"schema":"acio.input/1","request_id":"r","mode":"ask","instruction":{"text":"q","language":language},"workspace":{"root":"/w","access":"read_only"}});
    for ok in ["ko", "en", "en-US"] {
        assert_eq!(cli_io::validate("input", &input(ok)), Ok(()), "{ok}");
    }
    for bad in ["korean", "KO", "en-us", "e", "en-US-x"] {
        assert!(cli_io::validate("input", &input(bad)).is_err(), "{bad}");
    }
    let session = |ts: &str| ev(1, ts, "session", serde_json::json!({"session_id":"s"}));
    let result = |ts: &str| {
        ev(
            2,
            ts,
            "result",
            serde_json::json!({"status":"success","summary":"ok"}),
        )
    };
    // 10:00 at +09:00 is 01:00Z; the next event one second later in UTC is in order, one second earlier is not.
    let ordered = stream(&[
        session("2026-10-28T10:00:00+09:00"),
        result("2026-10-28T01:00:01.500Z"),
    ]);
    assert_eq!(
        cli_io::validate_stream("events", &ordered, Rules::default()),
        Ok(())
    );
    let backwards = stream(&[
        session("2026-10-28T10:00:00+09:00"),
        result("2026-10-27T23:59:59Z"),
    ]);
    assert!(
        cli_io::validate_stream("events", &backwards, Rules::default())
            .unwrap_err()
            .contains("ts")
    );
    let leap_day = stream(&[
        session("2028-02-29T00:00:00Z"),
        result("2028-03-01T00:00:00Z"),
    ]);
    assert_eq!(
        cli_io::validate_stream("events", &leap_day, Rules::default()),
        Ok(())
    );
    let not_leap = stream(&[
        session("2027-02-29T00:00:00Z"),
        result("2027-03-01T00:00:00Z"),
    ]);
    assert!(cli_io::validate_stream("events", &not_leap, Rules::default()).is_err());
}
