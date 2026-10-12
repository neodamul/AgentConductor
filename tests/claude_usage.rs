use agentconductor::quota;
#[test]
fn screen_reader_usage_reports_windows_without_confusing_stats_or_costs() {
    let text = "\x1b[2KSettings  Status Config Usage Stats\r\nTotal cost: $0.0000\nCurrent session\n100% 100% used\nResets 2:09pm (Asia/Seoul)\nCurrent week (all models)\n43% 43% used\nResets Oct 12 at 2:59am (Asia/Seoul)\nWhat's contributing to your limits usage?\n99% of your usage came from sessions active for 8+ hours\n";
    let value = quota::parse_claude_usage(text, 1000).unwrap();
    assert_eq!(value.windows.len(), 2);
    assert_eq!(value.windows[0].remaining_percent, Some(0.0));
    assert_eq!(value.windows[1].remaining_percent, Some(57.0));
    assert!(quota::parse_claude_usage("Permission Required: Accessing workspace", 1000).is_err());
    assert!(quota::parse_claude_usage("Current session\n143% used", 1000).is_err());
    assert!(quota::parse_claude_usage("Total cost: $1.00\nUsage: 100 input", 1000).is_err());
}

#[cfg(target_os = "macos")]
#[tokio::test]
#[ignore = "requires an existing authenticated Claude subscription and trusted repository"]
async fn authenticated_claude_usage_has_zero_generation_and_reaps_owned_process() {
    let root = std::env::temp_dir().join(format!(
        "ac-live-usage-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let profile = serde_json::from_value(
        serde_json::json!({"id":"claude","parser":"claude","executable":"claude","args":[]}),
    )
    .unwrap();
    let snapshot = quota::query_claude(
        &profile,
        &root,
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    )
    .await
    .unwrap();
    assert!(!snapshot.windows.is_empty());
    let raw = std::fs::read_to_string(root.join("stdout.terminal")).unwrap();
    let compact = raw.split_whitespace().collect::<String>();
    assert!(compact.contains("Totalduration(API):0s"));
    assert!(compact.contains("Totalcost:$0.0000"));
    let owner: agentconductor::process::Ownership =
        serde_json::from_slice(&std::fs::read(root.join("owner.json")).unwrap()).unwrap();
    for _ in 0..50 {
        if owner.ensure_quiescent().is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    owner.ensure_quiescent().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn explicit_enterprise_header_is_preserved_but_never_inferred_from_windows() {
    let text = "Claude Enterprise plan\nCurrent session\n27% used\nCurrent week\n64% used\n";
    let value = agentconductor::quota::parse_claude_usage(text, 1000).unwrap();
    assert_eq!(value.plan_type.as_deref(), Some("enterprise"));
    let plain = agentconductor::quota::parse_claude_usage(
        &text.replace("Claude Enterprise plan", "Claude Code"),
        1000,
    )
    .unwrap();
    assert!(plain.plan_type.is_none());
}

#[test]
fn first_run_and_trust_screens_stop_the_query_immediately() {
    // DD-94: screen reported from a real Windows machine; nothing is answered for the user.
    let onboarding = "Welcome to Claude Code v2.1.218\nLet's get started.\nChoose the text style that looks best with your terminal\nEnter selection [1-7], or Escape to cancel:";
    assert!(
        quota::claude_blocker(onboarding)
            .unwrap()
            .contains("first-run")
    );
    assert!(
        quota::claude_blocker("Permission Required: trust")
            .unwrap()
            .contains("trust")
    );
    assert!(quota::claude_blocker("Claude Code v2\n$\nCurrent session\n27% used").is_none());
}
