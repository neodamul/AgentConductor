//! Metadata collection must recognize interactive prompt variants without a generation request.
#[cfg(any(target_os = "macos", windows))]
#[tokio::test]
async fn claude_prompt_variants_collect_usage_and_reap_the_owned_process() {
    use agentconductor::{plan::AgentProfile, quota};
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
    #[cfg(target_os = "macos")]
    let _path = ScopedPath::set(&root);
    for (index, prompt) in ["❯", ">", "$"].iter().enumerate() {
        let executable = if cfg!(target_os = "macos") {
            "terminal".to_owned()
        } else {
            exe.to_string_lossy().into_owned()
        };
        let profile: AgentProfile = serde_json::from_value(serde_json::json!({"id":"claude","parser":"claude","executable":executable,"args":[],"environment":{"AC_QUOTA_PROMPT":prompt,"AC_QUOTA_CASE":if index==1{"trust_choice"}else{"normal"},"AC_QUOTA_ENTERPRISE":if index==1{"1"}else{"0"},"PATH":if cfg!(target_os = "macos"){"/definitely/missing"}else{""}}})).unwrap();
        #[cfg(target_os = "macos")]
        let profile = {
            let mut profile = profile;
            let term = ["dumb", "screen-256color", ""][index];
            profile.environment.insert("TERM".into(), term.into());
            profile.environment.insert(
                "AC_EXPECT_TERM".into(),
                if index == 1 { term } else { "xterm-256color" }.into(),
            );
            profile
        };
        let dir = root.join(index.to_string());
        let result = quota::query_claude(&profile, &dir, &root).await.unwrap();
        if index == 1 {
            assert_eq!(result.plan_type.as_deref(), Some("enterprise"));
            assert_eq!(result.budgets[0].remaining.as_deref(), Some("148.77"));
        } else {
            assert_eq!(result.windows[0].remaining_percent, Some(73.0));
        }
        let owner: agentconductor::process::Ownership =
            serde_json::from_slice(&std::fs::read(dir.join("owner.json")).unwrap()).unwrap();
        owner.ensure_quiescent().unwrap();
        assert!(
            std::fs::read_to_string(dir.join("stdout.terminal"))
                .unwrap()
                .contains(if index == 1 {
                    "spent"
                } else {
                    "Current session"
                })
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn enterprise_spent_allocation_preserves_exact_amounts_and_does_not_invent_reset_time() {
    // Synthetic amounts preserve the operator-reported output shape without publishing account usage.
    let text = "Claude Code v2.1.296\nEnterprise plan\n>\n70% used $351.23 / $500.00 spent - Resets Nov 1 (Asia/Seoul)";
    let parsed = agentconductor::quota::parse_claude_usage(text, 1791975600).unwrap();
    assert!(parsed.windows.is_empty());
    let budget = &parsed.budgets[0];
    assert_eq!(budget.used.as_deref(), Some("351.23"));
    assert_eq!(budget.allocated.as_deref(), Some("500.00"));
    assert_eq!(budget.remaining.as_deref(), Some("148.77"));
    assert!((budget.remaining_percent.unwrap() - 29.754).abs() < 0.000001);
    assert_eq!(
        budget.resets_at, None,
        "date-only reset must not become an invented time"
    );
    assert!(agentconductor::quota::parse_claude_usage("Total cost: $351.23", 1791975600).is_err());
    assert!(
        agentconductor::quota::parse_claude_usage("70% used $351.23 / $0.00 spent", 1791975600)
            .is_err()
    );
}

#[test]
fn banner_or_currency_without_an_input_prompt_does_not_send_usage() {
    for text in [
        "Claude Code v2.1.296",
        "Claude Code v2.1.296\nTotal cost: $0.00",
        ">",
        "Welcome\n❯",
        "Claude Code v2.1.296\n> Yes, I trust this folder",
    ] {
        assert!(!agentconductor::quota::claude_usage_ready(text));
    }
}

#[cfg(target_os = "macos")]
struct ScopedPath(Option<std::ffi::OsString>);

#[cfg(target_os = "macos")]
impl ScopedPath {
    fn set(path: &std::path::Path) -> Self {
        let previous = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", path) };
        Self(previous)
    }
}

#[cfg(target_os = "macos")]
impl Drop for ScopedPath {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            unsafe { std::env::set_var("PATH", path) };
        } else {
            unsafe { std::env::remove_var("PATH") };
        }
    }
}
