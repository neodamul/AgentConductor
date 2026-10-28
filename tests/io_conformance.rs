#![cfg(unix)]
//! GitHub #212: `agentconductor io conformance <exe>` runs a CLI through the acio contract and reports a pass/fail
//! table: it describes itself, answers a read-only question with a valid stream that ends in a success and exit 0,
//! and refuses an invalid input with exit code 2.
use agentconductor::cli_io::conformance::{self, Options, Outcome};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ac-conf-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.canonicalize().unwrap()
}
fn script(dir: &std::path::Path, body: &str) -> PathBuf {
    let path = dir.join("cli.sh");
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}
fn quick() -> Options {
    Options {
        describe_timeout: Duration::from_millis(600),
        run_timeout: Duration::from_secs(10),
    }
}
fn outcome<'a>(report: &'a conformance::Report, name: &str) -> &'a Outcome {
    &report
        .checks
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| {
            panic!(
                "no check {name}: {:?}",
                report.checks.iter().map(|c| &c.name).collect::<Vec<_>>()
            )
        })
        .outcome
}
const DESCRIBE: &str = r#"[ "$1" = --acio-describe ] && { printf '%s\n' '{"schema":"acio.describe/1","name":"fake","version":"1","acio":{"versions":[1]},"capabilities":{"read_only":true,"streaming":false},"invocation":{"input":"stdin"}}'; exit 0; }
"#;

#[tokio::test]
async fn the_minimal_example_passes_every_check() {
    let exe = std::fs::canonicalize("schemas/cli-io/v1/examples/minimal-cli.sh").unwrap();
    let report = conformance::run(&exe, &quick()).await;
    for check in &report.checks {
        assert!(
            matches!(check.outcome, Outcome::Pass),
            "{}: {:?}",
            check.name,
            check.outcome
        );
    }
    assert!(report.passed());
    let names: Vec<_> = report.checks.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["describe", "versions", "ask run", "invalid input"]);
}

#[tokio::test]
async fn a_stream_with_a_gap_and_a_cli_that_accepts_garbage_fail_with_reasons() {
    let d = dir("bad");
    let exe = script(
        &d,
        &format!(
            r#"{DESCRIBE}IFS= read -r input
case "$input" in *'"request_id"'*) ;; *) exit 0;; esac
printf '%s\n' '{{"schema":"acio.event/1","seq":1,"ts":"2026-10-28T01:00:00Z","type":"session","session_id":"s","data":{{"session_id":"s"}}}}'
printf '%s\n' '{{"schema":"acio.event/1","seq":3,"ts":"2026-10-28T01:00:01Z","type":"result","session_id":"s","data":{{"status":"success","summary":"x"}}}}'
"#
        ),
    );
    let report = conformance::run(&exe, &quick()).await;
    assert!(matches!(outcome(&report, "describe"), Outcome::Pass));
    match outcome(&report, "ask run") {
        Outcome::Fail(why) => assert!(why.contains("seq 3 should be 2"), "{why}"),
        other => panic!("{other:?}"),
    }
    match outcome(&report, "invalid input") {
        Outcome::Fail(why) => assert!(why.contains("exit code 2"), "{why}"),
        other => panic!("{other:?}"),
    }
    assert!(!report.passed());
    std::fs::remove_dir_all(d).unwrap();
}

#[tokio::test]
async fn a_cli_that_cannot_describe_itself_stops_after_the_first_check() {
    let d = dir("silent");
    let exe = script(&d, "sleep 30\n");
    let report = conformance::run(&exe, &quick()).await;
    assert!(
        matches!(outcome(&report, "describe"), Outcome::Fail(why) if why.contains("did not answer"))
    );
    assert!(
        matches!(outcome(&report, "ask run"), Outcome::Skip(_)),
        "later checks need the description"
    );
    assert!(!report.passed());
    std::fs::remove_dir_all(d).unwrap();
}

#[tokio::test]
async fn a_cli_without_read_only_support_skips_the_question() {
    let d = dir("write");
    let exe = script(
        &d,
        r#"[ "$1" = --acio-describe ] && { printf '%s\n' '{"schema":"acio.describe/1","name":"w","version":"1","acio":{"versions":[1]},"capabilities":{"read_only":false,"streaming":false},"invocation":{"input":"stdin"}}'; exit 0; }
exit 2
"#,
    );
    let report = conformance::run(&exe, &quick()).await;
    assert!(matches!(outcome(&report, "ask run"), Outcome::Skip(why) if why.contains("read-only")));
    assert!(matches!(outcome(&report, "invalid input"), Outcome::Pass));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn the_command_prints_the_table_and_its_exit_status_follows_the_result() {
    let exe = env!("CARGO_BIN_EXE_agentconductor");
    let good = std::fs::canonicalize("schemas/cli-io/v1/examples/minimal-cli.sh").unwrap();
    let out = std::process::Command::new(exe)
        .args(["io", "conformance", good.to_str().unwrap()])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("pass") && text.contains("ask run") && text.contains("4 of 4 checks passed"),
        "{text}"
    );
    let d = dir("cmd");
    let bad = script(&d, "sleep 30\n");
    let out = std::process::Command::new(exe)
        .args(["io", "conformance", bad.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("FAIL"));
    std::fs::remove_dir_all(d).unwrap();
}
