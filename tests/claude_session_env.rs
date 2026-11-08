//! GitHub #585: the Claude usage lookup drops the parent's Claude Code session markers and keeps user auth.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn the_lookup_does_not_inherit_a_claude_code_session() {
    use agentconductor::{plan::AgentProfile, quota};
    use std::time::Duration;
    // Single test in this process: nothing else reads the environment while it is changed.
    unsafe {
        std::env::set_var("CLAUDECODE", "1");
        std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "cli");
        std::env::set_var("CLAUDE_CODE_OAUTH_TOKEN", "kept-for-auth");
    }
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let exe = root.join("terminal");
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/terminal_quota_agent.rs", "-o"])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let profile: AgentProfile = serde_json::from_value(serde_json::json!({"id":"claude","parser":"claude","executable":exe,"args":[],"environment":{"AC_QUOTA_REQUIRE_CLEAN_SESSION":"1"}})).unwrap();
    let snapshot =
        quota::query_claude_within(&profile, &root.join("q"), &root, Duration::from_secs(30))
            .await
            .expect("session markers are not passed on and the OAuth token is");
    assert_eq!(snapshot.windows[0].remaining_percent, Some(73.0));
    std::fs::remove_dir_all(root).unwrap();
}
