use agentconductor::cli_io;
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
    if path.extension().is_some_and(|ext| ext == "jsonl") {
        text.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    } else {
        vec![serde_json::from_str(&text).unwrap()]
    }
}

fn examples(folder: &str) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(format!("{DIR}/{folder}"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    paths
}

#[test]
fn embedded_schemas_match_the_versioned_assets() {
    for kind in cli_io::KINDS {
        let schema = cli_io::schema(kind).unwrap();
        assert_eq!(
            schema,
            fs::read_to_string(format!("{DIR}/{kind}.schema.json")).unwrap()
        );
        assert!(serde_json::from_str::<serde_json::Value>(schema).is_ok());
    }
    assert!(cli_io::schema("unknown").is_err());
}

#[test]
fn valid_and_invalid_protocol_examples_follow_their_schemas() {
    for path in examples("examples/valid").into_iter().filter(|path| {
        matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("json" | "jsonl")
        )
    }) {
        for document in documents(&path) {
            assert_eq!(
                cli_io::validate(
                    kind_of(path.file_name().unwrap().to_str().unwrap()),
                    &document
                ),
                Ok(()),
                "{path:?}"
            );
        }
    }
    for path in examples("examples/invalid") {
        let kind = kind_of(path.file_name().unwrap().to_str().unwrap());
        assert!(
            documents(&path)
                .iter()
                .any(|document| cli_io::validate(kind, document).is_err()),
            "{path:?} should be rejected"
        );
    }
}

#[test]
fn built_in_agent_presets_validate_as_agent_settings() {
    for provider in ["claude", "codex", "opencode"] {
        let document: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(format!("{DIR}/presets/{provider}.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(cli_io::validate("agent", &document), Ok(()), "{provider}");
    }
}

#[test]
fn schema_errors_report_the_failing_pointer_and_constraint() {
    let event = serde_json::json!({"schema":"acio.event/1","seq":0,"ts":"2026-10-28T01:00:00Z","type":"progress","data":{"message":"x"}});
    let errors = cli_io::validate("event", &event).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.starts_with("/seq:") && error.contains("minimum"))
    );
}

#[test]
fn input_accepts_a_bounded_supervisor_assigned_session_id() {
    let input = serde_json::json!({
        "schema":"acio.input/1",
        "request_id":"r1",
        "mode":"task",
        "instruction":{"text":"work"},
        "session":{"id":"session-1","resume_id":"session-1"}
    });
    assert_eq!(cli_io::validate("input", &input), Ok(()));
    let mut invalid = input;
    invalid["session"]["id"] = "".into();
    assert!(cli_io::validate("input", &invalid).is_err());
}
