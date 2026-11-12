use super::*;
#[test]
fn timeout_names_the_step_and_reference_without_dumping_provider_json() {
    let reference = Reference {
        kind: "jira".into(),
        id: "AC-1".into(),
    };
    let error = step_error(
        "coordinator failed (timed out): parent_tool_use_id:null private-payload".into(),
        "selection",
        &reference,
        Path::new("local-preview"),
    );
    assert!(
        error.contains("jira:AC-1") && error.contains("300") && error.contains("local-preview")
    );
    assert!(!error.contains("private-payload") && !error.contains("parent_tool_use_id"));
    let summary = step_error(
        "coordinator failed (timed out)".into(),
        "summary",
        &reference,
        Path::new("local-summary"),
    );
    assert_ne!(error, summary);
    assert!(summary.contains("local-summary"));
    assert_eq!(
        step_error(
            "coordinator failed (exit code1): denied".into(),
            "selection",
            &reference,
            Path::new("local-preview")
        ),
        "coordinator failed (exit code1): denied"
    );
}

#[tokio::test]
async fn an_overall_step_deadline_cancels_the_owned_cli_without_late_output() {
    let root = std::env::temp_dir().join(format!(
        "ac-mcp-budget-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let marker = root.join("late.txt");
    let script = format!(
        "import time,pathlib;time.sleep(0.5);pathlib.Path({:?}).write_text('late')",
        marker.to_string_lossy()
    );
    let profile:crate::plan::AgentProfile=serde_json::from_value(serde_json::json!({"id":"slow","executable":if cfg!(windows){"python"}else{"python3"},"args":["-c",script],"parser":"custom","permission_mode":"read_only"})).unwrap();
    let reference = Reference {
        kind: "jira".into(),
        id: "AC-1".into(),
    };
    let expired = root.join("expired");
    let error = run_step(
        &profile,
        &root,
        &expired,
        "{}",
        "selection",
        &reference,
        Duration::ZERO,
    )
    .await
    .unwrap_err();
    assert!(
        error.contains("AC-1") && !expired.exists(),
        "expired work must not start a CLI"
    );
    let error = run_step(
        &profile,
        &root,
        &root.join("attempt"),
        "{}",
        "selection",
        &reference,
        Duration::from_millis(100),
    )
    .await
    .unwrap_err();
    assert!(error.contains("AC-1") && error.contains("300"));
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert!(!marker.exists(), "timed-out CLI survived cancellation");
    std::fs::remove_dir_all(root).unwrap();
}
