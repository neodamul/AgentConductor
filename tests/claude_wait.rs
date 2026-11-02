//! GitHub #297 (R-201 / QAS-199): a Claude CLI that is slow to start is waited for, a stuck one is
//! given up with the limit named, and the default limit is three minutes for both collectors.
#[cfg(any(target_os = "macos", windows))]
#[tokio::test]
async fn a_slow_cli_is_waited_for_and_a_stuck_one_names_the_limit() {
    use agentconductor::{plan::AgentProfile, quota};
    use std::time::Duration;
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let exe = root.join(if cfg!(windows) {
        "terminal.exe"
    } else {
        "terminal"
    });
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/terminal_quota_agent.rs", "-o"])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let profile = |delay_ms: u64| -> AgentProfile {
        serde_json::from_value(serde_json::json!({"id":"claude","parser":"claude","executable":exe,"args":[],"environment":{"AC_QUOTA_DELAY_MS":delay_ms.to_string()}})).unwrap()
    };
    let slow = quota::query_claude_within(
        &profile(3000),
        &root.join("slow"),
        &root,
        Duration::from_secs(15),
    )
    .await
    .unwrap();
    assert_eq!(slow.windows[0].remaining_percent, Some(73.0));
    let error = quota::query_claude_within(
        &profile(5000),
        &root.join("stuck"),
        &root,
        Duration::from_secs(2),
    )
    .await
    .unwrap_err();
    assert!(error.contains("before /usage"), "{error}");
    assert!(error.contains("after 2 s"), "{error}");
    // GitHub #295: a screen that was read but not understood is named, not hidden behind the timeout.
    let unread = AgentProfile {
        environment: [("AC_QUOTA_ENTERPRISE".to_string(), "zero".to_string())].into(),
        ..profile(0)
    };
    let error =
        quota::query_claude_within(&unread, &root.join("unread"), &root, Duration::from_secs(3))
            .await
            .unwrap_err();
    assert!(error.contains("after /usage"), "{error}");
    assert!(
        error.contains("allocated budget must be positive"),
        "{error}"
    );
    // GitHub #317: an expired or missing login is named at once instead of waiting for the limit.
    let signed_out = AgentProfile {
        environment: [("AC_QUOTA_CASE".to_string(), "signed_out".to_string())].into(),
        ..profile(0)
    };
    let started = std::time::Instant::now();
    let error = quota::query_claude_within(
        &signed_out,
        &root.join("signed-out"),
        &root,
        Duration::from_secs(30),
    )
    .await
    .unwrap_err();
    assert!(
        error.contains("not signed in or the login expired"),
        "{error}"
    );
    assert!(started.elapsed() < Duration::from_secs(20));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_default_limit_is_three_minutes_for_both_collectors() {
    assert_eq!(
        agentconductor::quota::CLAUDE_WAIT,
        std::time::Duration::from_secs(180)
    );
    for source in [
        include_str!("../src/quota/terminal.rs"),
        include_str!("../src/quota/terminal/windows.rs"),
    ] {
        assert!(
            !source.contains("from_secs(20)"),
            "a literal 20 s limit is left"
        );
    }
}
