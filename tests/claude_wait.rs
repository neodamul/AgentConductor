//! GitHub #297: Claude usage waits are bounded, stage-aware, and testable.

#[cfg(any(target_os = "macos", windows))]
struct TempRoot(std::path::PathBuf);

#[cfg(any(target_os = "macos", windows))]
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(any(target_os = "macos", windows))]
#[tokio::test]
async fn waits_for_slow_start_and_names_both_timeout_stages() {
    use agentconductor::{plan::AgentProfile, quota};
    use std::time::Duration;

    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let _cleanup = TempRoot(root.clone());
    let executable = root.join(if cfg!(windows) {
        "terminal.exe"
    } else {
        "terminal"
    });
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/terminal_quota_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let profile = |environment: &[(&str, &str)]| -> AgentProfile {
        let environment = environment
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect::<std::collections::BTreeMap<_, _>>();
        serde_json::from_value(serde_json::json!({
            "id":"claude","parser":"claude","executable":executable.clone(),
            "args":[],"environment":environment
        }))
        .unwrap()
    };

    let result = quota::query_claude_within(
        &profile(&[("AC_QUOTA_DELAY_MS", "2500")]),
        &root.join("slow"),
        &root,
        Duration::from_secs(4),
    )
    .await
    .unwrap();
    assert_eq!(result.windows[0].remaining_percent, Some(73.0));

    let before_usage = quota::query_claude_within(
        &profile(&[("AC_QUOTA_DELAY_MS", "3000")]),
        &root.join("before-usage"),
        &root,
        Duration::from_secs(1),
    )
    .await
    .unwrap_err();
    assert!(before_usage.contains("before /usage"), "{before_usage}");
    assert!(before_usage.contains("after 1 s"), "{before_usage}");

    let after_usage = quota::query_claude_within(
        &profile(&[("AC_QUOTA_HANG_AFTER_USAGE", "1")]),
        &root.join("after-usage"),
        &root,
        Duration::from_secs(1),
    )
    .await
    .unwrap_err();
    assert!(after_usage.contains("after /usage"), "{after_usage}");
    assert!(after_usage.contains("after 1 s"), "{after_usage}");
}

#[test]
fn production_wait_is_one_minute_for_both_collectors() {
    assert_eq!(
        agentconductor::quota::CLAUDE_WAIT,
        std::time::Duration::from_secs(60)
    );
    for source in [
        include_str!("../src/quota/terminal/query.rs"),
        include_str!("../src/quota/terminal/windows.rs"),
    ] {
        assert!(
            !source.contains("from_secs(20)"),
            "20-second timeout remains"
        );
    }
}
