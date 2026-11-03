#![cfg(unix)]
//! GitHub #208: a CLI that speaks acio runs under the supervisor with a live control channel: tool requests wait
//! for a decision (denied when none comes), the supervisor can send a user message and a cancel, a protocol
//! violation ends the run, and `--acio-describe` registers a CLI. Decoding needs no mapping tables.
use agentconductor::acio::{self, Decision};
use agentconductor::plan::AgentProfile;
use agentconductor::process::ProcessSpec;
use agentconductor::protocol::{Decoder, EventKind};
use std::{os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};

fn root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ac-acio-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.canonicalize().unwrap()
}
fn script(dir: &std::path::Path, body: &str) -> PathBuf {
    let path = dir.join("cli.sh");
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}
fn spec(dir: &std::path::Path, exe: PathBuf) -> ProcessSpec {
    ProcessSpec {
        executable: exe,
        args: vec![],
        cwd: dir.to_path_buf(),
        input: acio::input_line(
            &serde_json::json!({"schema":"acio.input/1","request_id":"r1","mode":"task","instruction":{"text":"go"},"workspace":{"root":dir,"access":"workspace_write","paths":["a.txt"]},"control":{"enabled":true,"decision_timeout_ms":1500}}),
        ),
        log_dir: dir.join("attempt"),
        timeout: Duration::from_secs(20),
        signal_grace: Duration::from_millis(500),
        environment: Default::default(),
    }
}
// Emits an event line with the next seq.
const EMIT: &str = r#"n=0
emit() { n=$((n+1)); printf '{"schema":"acio.event/1","seq":%s,"ts":"2026-10-28T01:00:%02dZ","session_id":"s","type":"%s","data":%s}\n' "$n" "$n" "$1" "$2"; }
IFS= read -r input
emit session '{"session_id":"s"}'
"#;

#[tokio::test]
async fn a_tool_request_waits_for_the_decision_and_the_cli_sees_it() {
    let dir = root("decision");
    let exe = script(
        &dir,
        &format!(
            r#"{EMIT}emit tool '{{"call_id":"c1","name":"shell","status":"requested","needs_decision":true}}'
IFS= read -r control
case "$control" in *'"decision":"allow"'*) emit tool '{{"call_id":"c1","name":"shell","status":"completed"}}';; *) emit tool '{{"call_id":"c1","name":"shell","status":"denied"}}';; esac
emit result '{{"status":"success","summary":"done"}}'
"#
        ),
    );
    let seen = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let log = seen.clone();
    let decide = move |request: acio::ToolRequest| {
        log.lock().unwrap().push(request.name.clone());
        async move { Ok(Decision::Allow) }
    };
    let run = acio::run(spec(&dir, exe), decide, acio::Options::default())
        .await
        .unwrap();
    assert_eq!(*seen.lock().unwrap(), ["shell"]);
    let stdout = std::fs::read_to_string(&run.result.stdout).unwrap();
    assert!(stdout.contains(r#""status":"completed""#), "{stdout}");
    assert_eq!(
        agentconductor::cli_io::validate_stream("events", &stdout, Default::default()),
        Ok(())
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn no_decision_in_time_means_denied() {
    let dir = root("timeout");
    let exe = script(
        &dir,
        &format!(
            r#"{EMIT}emit tool '{{"call_id":"c1","name":"shell","status":"requested","needs_decision":true}}'
IFS= read -r control
case "$control" in *'"decision":"allow"'*) : > accidentally-allowed; emit tool '{{"call_id":"c1","name":"shell","status":"completed"}}';; *) emit tool '{{"call_id":"c1","name":"shell","status":"denied"}}';; esac
emit result '{{"status":"success","summary":"done"}}'
"#
        ),
    );
    let slow = |_: acio::ToolRequest| async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(Decision::Allow)
    };
    let error = acio::run(spec(&dir, exe), slow, acio::Options::default())
        .await
        .err()
        .unwrap();
    assert!(error.contains("no decision in time"), "{error}");
    assert!(!dir.join("accidentally-allowed").exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn a_user_message_and_a_cancel_reach_the_cli_in_order() {
    let dir = root("outbound");
    let exe = script(
        &dir,
        &format!(
            r#"{EMIT}emit progress '{{"message":"working"}}'
IFS= read -r first
IFS= read -r second
case "$first" in *'"seq":1'*'"user_message"'*|*'"user_message"'*'"seq":1'*) emit ack '{{"control_seq":1,"status":"applied"}}';; *) emit ack '{{"control_seq":1,"status":"ignored"}}';; esac
case "$second" in *'"cancel"'*) emit ack '{{"control_seq":2,"status":"applied"}}'; emit result '{{"status":"cancelled","summary":"stopped"}}';; *) emit result '{{"status":"failure","summary":"no cancel"}}';; esac
"#
        ),
    );
    let handle = acio::Control::new();
    let sender = handle.clone();
    let task = tokio::spawn(acio::run(
        spec(&dir, exe),
        |_| async { Ok(Decision::Deny) },
        acio::Options {
            control: Some(handle),
            ..Default::default()
        },
    ));
    tokio::time::sleep(Duration::from_millis(300)).await;
    sender.user_message("skip lint").unwrap();
    sender.cancel("operator stopped").unwrap();
    let run = task.await.unwrap().unwrap();
    let stdout = std::fs::read_to_string(&run.result.stdout).unwrap();
    assert!(
        stdout.contains(r#""control_seq":1,"status":"applied""#),
        "{stdout}"
    );
    assert!(stdout.contains(r#""status":"cancelled""#), "{stdout}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn a_protocol_violation_ends_the_run() {
    let dir = root("violation");
    let exe = script(
        &dir,
        &format!(
            r#"{EMIT}printf '%s\n' '{{"schema":"acio.event/1","seq":9,"ts":"2026-10-28T01:00:09Z","type":"progress","data":{{"message":"skipped seqs"}}}}'
sleep 5
"#
        ),
    );
    let error = acio::run(
        spec(&dir, exe),
        |_| async { Ok(Decision::Deny) },
        acio::Options::default(),
    )
    .await
    .err()
    .unwrap();
    assert!(error.contains("seq 9 should be 2"), "{error}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn describe_registers_a_cli_and_a_silent_one_times_out() {
    let dir = root("describe");
    let good = script(
        &dir,
        "[ \"$1\" = --acio-describe ] && printf '%s\\n' '{\"schema\":\"acio.describe/1\",\"name\":\"fake\",\"version\":\"1\",\"acio\":{\"versions\":[1]},\"capabilities\":{\"read_only\":true,\"streaming\":false},\"invocation\":{\"input\":\"stdin\"}}'\n",
    );
    let described = acio::describe(&good, Duration::from_secs(5)).await.unwrap();
    assert_eq!(described["name"], "fake");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["io", "describe", good.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("\"name\":\"fake\""));
    let hang = script(&dir, "sleep 30\n");
    let error = acio::describe(&hang, Duration::from_millis(400))
        .await
        .unwrap_err();
    assert!(error.contains("did not answer"), "{error}");
    let invalid = script(&dir, "echo '{\"schema\":\"acio.describe/1\"}'\n");
    assert!(
        acio::describe(&invalid, Duration::from_secs(5))
            .await
            .unwrap_err()
            .contains("required")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

fn profile(extra: serde_json::Value) -> Result<AgentProfile, String> {
    let mut base = serde_json::json!({"id":"inhouse","executable":"/bin/inhouse","args":["run"],"parser":"custom","protocol":"acio"});
    base.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let profile: AgentProfile = serde_json::from_value(base).map_err(|e| e.to_string())?;
    profile.validate_custom_configuration()?;
    Ok(profile)
}

#[test]
fn the_acio_decoder_needs_no_mapping_tables() {
    let profile = profile(serde_json::json!({})).unwrap();
    let mut decoder = Decoder::for_profile(&profile, 1 << 20);
    let text =
        std::fs::read_to_string("schemas/cli-io/v1/examples/valid/events-chat.jsonl").unwrap();
    let events = decoder.push(text.as_bytes()).unwrap();
    let kinds: Vec<_> = events.iter().map(|e| e.kind).collect();
    use EventKind::*;
    assert_eq!(
        kinds,
        [Session, Progress, Progress, Progress, Usage, Result]
    );
    assert_eq!(events[0].session_id.as_deref(), Some("s-1"));
    let failure = r#"{"schema":"acio.event/1","seq":1,"ts":"2026-10-28T01:00:00Z","type":"result","data":{"status":"failure","summary":"x"}}"#;
    assert_eq!(
        Decoder::for_profile(&profile, 1 << 20)
            .push(format!("{failure}\n").as_bytes())
            .unwrap()[0]
            .kind,
        Error
    );
    let unknown = r#"{"schema":"acio.event/1","seq":1,"ts":"2026-10-28T01:00:00Z","type":"x-inhouse-audit","data":{}}"#;
    assert_eq!(
        Decoder::for_profile(&profile, 1 << 20)
            .push(format!("{unknown}\n").as_bytes())
            .unwrap()[0]
            .kind,
        Unknown
    );
}

#[test]
fn the_acio_protocol_is_a_custom_profile_setting_with_no_mapping_of_its_own() {
    assert!(profile(serde_json::json!({})).is_ok());
    assert!(
        profile(serde_json::json!({"event_types":{"done":"result"}}))
            .unwrap_err()
            .contains("protocol")
    );
    assert!(
        profile(serde_json::json!({"result_pointer":"/x"}))
            .unwrap_err()
            .contains("protocol")
    );
    assert!(
        profile(serde_json::json!({"parser":"claude"}))
            .unwrap_err()
            .contains("custom")
    );
}
