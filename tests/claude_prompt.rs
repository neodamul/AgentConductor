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
    for (index, prompt) in ["❯", ">", "$"].iter().enumerate() {
        let profile: AgentProfile = serde_json::from_value(serde_json::json!({"id":"claude","parser":"claude","executable":exe,"args":[],"environment":{"AC_QUOTA_PROMPT":prompt,"AC_QUOTA_CASE":if index==1{"trust_choice"}else{"normal"},"AC_QUOTA_ENTERPRISE":if index==1{"1"}else{"0"}}})).unwrap();
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

#[test]
fn enterprise_amounts_with_thousands_separators_and_cursor_spacing_are_read() {
    // GitHub #295: a company allocation over $999 is written with commas, and a terminal that draws
    // spaces as cursor-forward moves leaves no whitespace between the words.
    let commas = "Enterprise plan\n>\n70% used $1,351.23 / $5,000.00 spent - Resets Nov 1";
    let budget = &agentconductor::quota::parse_claude_usage(commas, 1791975600)
        .unwrap()
        .budgets[0];
    assert_eq!(budget.used.as_deref(), Some("1351.23"));
    assert_eq!(budget.allocated.as_deref(), Some("5000.00"));
    assert_eq!(budget.remaining.as_deref(), Some("3648.77"));
    let cursor = "Enterprise plan\r\n>\r\n70%\x1b[1Cused\x1b[1C$351.23\x1b[1C/\x1b[1C$500.00\x1b[1Cspent\x1b[1C-\x1b[1CResets\x1b[1CNov\x1b[1C1";
    let budget = &agentconductor::quota::parse_claude_usage(cursor, 1791975600)
        .unwrap()
        .budgets[0];
    assert_eq!(budget.allocated.as_deref(), Some("500.00"));
    for bad in ["$1,35.2", "$,500", "$5,00,0.00"] {
        let text = format!("70% used $1.00 / {bad} spent");
        assert!(
            agentconductor::quota::parse_claude_usage(&text, 1791975600).is_err(),
            "{bad}"
        );
    }
}
