//! GitHub #207: the acio schemas ship in the binary, a small checker validates documents against them, and the
//! stream rules JSON Schema cannot express are checked on a whole captured stream.
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
    for name in ["claude", "codex", "opencode"] {
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
