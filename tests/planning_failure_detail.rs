//! A failed planning step shows its cause, not only a log folder.
use agentconductor::{coordinator, process::ProcessResult};

fn result(dir: &std::path::Path, stderr: &str, stdout: &str, exit: Option<i32>) -> ProcessResult {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("stderr.log"), stderr).unwrap();
    std::fs::write(dir.join("stdout.jsonl"), stdout).unwrap();
    ProcessResult {
        exit_code: exit,
        termination_signal: None,
        timed_out: false,
        stopped: false,
        stdout: dir.join("stdout.jsonl"),
        stderr: dir.join("stderr.log"),
    }
}

#[test]
fn a_failed_coordinator_reports_its_status_and_own_error_output() {
    let dir = std::env::temp_dir().join(format!(
        "ac-plan-failure-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let failed = result(
        &dir.join("a"),
        "Error: model not found: gpt-x\n",
        "",
        Some(2),
    );
    let text = coordinator::failure("planning coordinator", &failed);
    assert!(text.contains("exit code 2"), "{text}");
    assert!(text.contains("model not found: gpt-x"), "{text}");
    assert!(text.contains("logs:"), "{text}");

    // Without stderr, the last output line (an error event) is the cause.
    let event = result(
        &dir.join("b"),
        "",
        "{\"type\":\"turn.started\"}\n{\"type\":\"error\",\"message\":\"usage limit reached\"}\n",
        Some(1),
    );
    let text = coordinator::failure("planning coordinator", &event);
    assert!(text.contains("usage limit reached"), "{text}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_plan_review_shows_the_error_text() {
    let script = include_str!("../src/ui/startup.js");
    assert!(script.contains("'plan-failure-detail'"));
    assert!(script.contains("job.error"));
}
