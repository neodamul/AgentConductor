use agentconductor::ui::request_diagnostics;
use serde_json::Value;
use std::{fs, time::Duration};

#[test]
fn logs_all_chat_requests_without_recording_request_contents() {
    let path = temp_log("chat");
    assert!(request_diagnostics::record_at(
        &path,
        "chat",
        12,
        42,
        Duration::from_millis(5),
        true,
    ));
    let records = read_records(&path);
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["event"], "start");
    assert_eq!(records[0]["kind"], "chat");
    assert_eq!(records[0]["at_unix_ms"], 42);
    assert_eq!(records[0]["bytes"], 12);
    assert_eq!(records[1]["event"], "done");
    assert_eq!(records[1]["duration_ms"], 5);
    assert_eq!(records[1]["outcome"], "ok");
    assert!(!fs::read_to_string(&path).unwrap().contains("secret"));
    remove_log(&path);
}

#[test]
fn logs_non_chat_requests_only_after_one_second() {
    let path = temp_log("fast");
    assert!(!request_diagnostics::record_at(
        &path,
        "snapshot",
        10,
        42,
        Duration::from_millis(999),
        true,
    ));
    assert!(!path.exists());
    assert!(request_diagnostics::record_at(
        &path,
        "snapshot",
        10,
        42,
        Duration::from_secs(1),
        false,
    ));
    let records = read_records(&path);
    assert_eq!(records.len(), 2);
    assert_eq!(records[1]["outcome"], "error");
    remove_log(&path);
}

#[test]
fn rotates_the_log_at_the_size_limit() {
    let path = temp_log("rotate");
    fs::write(&path, vec![b'x'; request_diagnostics::MAX_BYTES]).unwrap();
    assert!(request_diagnostics::record_at(
        &path,
        "chat",
        1,
        42,
        Duration::from_millis(1),
        true,
    ));
    assert!(path.with_extension("log.1").metadata().unwrap().len() > 0);
    assert!(fs::metadata(&path).unwrap().len() < request_diagnostics::MAX_BYTES as u64);
    remove_log(&path);
}

#[test]
fn diagnostics_write_failure_does_not_change_the_request_result() {
    let success = request_diagnostics::dispatch_with(
        br#"{"kind":"chat"}"#,
        || Ok("accepted"),
        |_, _, _, _, _| Err("disk unavailable"),
    );
    assert_eq!(success.unwrap(), "accepted");

    let failure = request_diagnostics::dispatch_with::<(), _>(
        br#"{"kind":"chat"}"#,
        || Err("original request error".into()),
        |_, _, _, _, _| Err("disk unavailable"),
    );
    assert_eq!(failure.unwrap_err(), "original request error");
}

fn temp_log(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ac-request-log-{label}-{}-{}.log",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn read_records(path: &std::path::Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn remove_log(path: &std::path::Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("log.1"));
}
