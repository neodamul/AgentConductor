//! GitHub #209: the argument building for the known agent CLIs is recorded as a golden file, so moving it from
//! code into agent settings (presets as data) provably changes nothing. `AC_UPDATE_GOLDEN=1` rewrites the file.
use agentconductor::{
    adapters::{self, InputMode},
    plan::AgentProfile,
    protocol::Provider,
};
use serde_json::{Value, json};
#[path = "support/env.rs"]
mod env;

const GOLDEN_DIR: &str = "tests/data";

fn mode(m: InputMode) -> &'static str {
    match m {
        InputMode::Stdin => "stdin",
        InputMode::File => "file",
    }
}
fn invocation(p: Provider, session: Option<&str>) -> Value {
    match adapters::invocation(p, session, "task.txt") {
        Ok(i) => json!({"executable":i.executable,"args":i.args,"input":mode(i.input)}),
        Err(e) => json!({"error":e}),
    }
}
fn profile(parser: &str, args: Vec<String>) -> AgentProfile {
    serde_json::from_value(json!({
        "id":"a","executable":"/bin/agent","args":args,"parser":parser,
        "catalog":{"source":"fixture","models":["m1"],"queried_at":0,"cli_version":"1"}
    }))
    .unwrap()
}
fn session_args(p: &AgentProfile, model: Option<&str>, session: Option<&str>) -> Value {
    match adapters::session_args(p, model, session, "task file.md") {
        Ok(a) => json!(a),
        Err(e) => json!({"error":e}),
    }
}

fn snapshot() -> Value {
    let providers = [
        ("claude", Provider::Claude),
        ("codex", Provider::Codex),
        ("opencode", Provider::OpenCode),
        ("custom", Provider::Custom),
    ];
    let mut out = serde_json::Map::new();
    for (name, p) in providers {
        let mut entry = serde_json::Map::new();
        entry.insert("invocation".into(), invocation(p, None));
        entry.insert("invocation_resume".into(), invocation(p, Some("sess-1")));
        entry.insert(
            "invocation_bad_session".into(),
            invocation(p, Some("--last")),
        );
        entry.insert(
            "full_access_args".into(),
            json!(adapters::full_access_args(p)),
        );
        for (label, args) in [
            (
                "plain",
                vec!["exec".to_string(), "-".into(), "--json".into()],
            ),
            (
                "with_skip",
                vec!["exec".into(), "--skip-git-repo-check".into()],
            ),
            ("no_exec", vec!["-p".into()]),
        ] {
            entry.insert(
                format!("direct_folder_{label}"),
                match adapters::direct_folder_args(p, args) {
                    Ok(a) => json!(a),
                    Err(e) => json!({"error":e}),
                },
            );
        }
        // Session args over the initial template and, where one exists, the full-access template.
        let initial = adapters::invocation(p, None, "{task_file}")
            .map(|i| i.args)
            .unwrap_or_else(|_| vec!["run".into(), "{task_file}".into(), "{model}".into()]);
        let mut templates = vec![("initial", initial)];
        if let Some(full) = adapters::full_access_args(p) {
            templates.push(("full_access", full));
        }
        for (label, args) in templates {
            let profile = profile(name, args);
            for (case, model, session) in [
                ("none", None, None),
                ("model", Some("m1"), None),
                ("session", None, Some("sess-1")),
                ("both", Some("m1"), Some("sess-1")),
                ("bad_session", None, Some("a b")),
                ("bad_model", Some("zz"), None),
            ] {
                entry.insert(
                    format!("session_{label}_{case}"),
                    session_args(&profile, model, session),
                );
            }
        }
        out.insert(name.into(), Value::Object(entry));
    }
    // Ambiguous selectors and a foreign template are refused as before.
    let ambiguous = profile("claude", vec!["-p".into(), "--resume".into(), "x".into()]);
    out.insert(
        "claude_ambiguous_session".into(),
        session_args(&ambiguous, None, Some("sess-1")),
    );
    let duplicate = profile("claude", vec!["-p".into(), "--model".into(), "m1".into()]);
    out.insert(
        "claude_duplicate_model".into(),
        session_args(&duplicate, Some("m1"), None),
    );
    let foreign = profile("codex", vec!["exec".into(), "--other".into()]);
    out.insert(
        "codex_foreign_template_resume".into(),
        session_args(&foreign, None, Some("sess-1")),
    );
    Value::Object(out)
}

#[test]
fn argument_building_for_the_known_clis_matches_the_golden_file() {
    // One error text comes from the UI message catalog; a saved interface language must not leak into the golden file.
    let home = std::env::temp_dir().join(format!("ac-golden-home-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let mut env = env::lock();
    env.set("HOME", &home);
    env.set("USERPROFILE", &home);
    let now = snapshot();
    let entries = now.as_object().unwrap();
    if std::env::var_os("AC_UPDATE_GOLDEN").is_some() {
        for (key, value) in entries {
            let path = format!("{GOLDEN_DIR}/adapter_golden_{key}.json");
            let document = serde_json::json!({key: value});
            std::fs::write(
                path,
                serde_json::to_string_pretty(&document).unwrap() + "\n",
            )
            .unwrap();
        }
    }
    let mut golden = serde_json::Map::new();
    for entry in std::fs::read_dir(GOLDEN_DIR).unwrap() {
        let path = entry.unwrap().path();
        if path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("adapter_golden_")
        {
            let document: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            golden.extend(document.as_object().unwrap().clone());
        }
    }
    let golden = Value::Object(golden);
    assert_eq!(now, golden);
    drop(env);
    let _ = std::fs::remove_dir_all(home);
}
