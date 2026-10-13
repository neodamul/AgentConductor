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

#[test]
fn per_model_week_never_overwrites_the_all_models_week() {
    // Screen captured from Claude Code 2.1.296 on Windows (Team plan).
    let text = "Current session\n1% 1% used\nResets 7:20pm (Asia/Seoul)\nCurrent week (all models)\n1% 1% used\nResets Oct 15, 12am (Asia/Seoul)\nCurrent week (Fable)\n0% 0% used\nCurrent week (Opus)\n5% 5% used\nWhat's contributing to your limits usage?\n";
    let value = quota::parse_claude_usage(text, 1000).unwrap();
    let used = |label: &str| {
        value
            .windows
            .iter()
            .find(|w| w.label == label)
            .and_then(|w| w.used_percent)
    };
    assert_eq!(value.windows.len(), 4);
    assert_eq!(used("five_hour"), Some(1.0));
    assert_eq!(used("seven_day"), Some(1.0));
    assert_eq!(used("seven_day_fable"), Some(0.0));
    assert_eq!(used("seven_day_opus"), Some(5.0));
}

#[cfg(any(target_os = "macos", windows))]
#[tokio::test]
async fn empty_project_folder_is_reported_before_any_process_starts() {
    let profile = serde_json::from_value(
        serde_json::json!({"id":"claude","parser":"claude","executable":"claude","args":[]}),
    )
    .unwrap();
    let dir = std::env::temp_dir().join(format!(
        "ac-empty-cwd-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let error = quota::query_claude(&profile, &dir, std::path::Path::new(""))
        .await
        .unwrap_err();
    assert!(error.contains("project folder"), "{error}");
    assert!(!dir.exists());
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
fn first_run_stops_the_query_and_trust_is_recognised() {
    // DD-94: screen reported from a real Windows machine; first-run setup is not answered.
    let onboarding = "Welcome to Claude Code v2.1.218\nLet's get started.\nChoose the text style that looks best with your terminal\nEnter selection [1-7], or Escape to cancel:";
    assert!(
        quota::claude_blocker(onboarding)
            .unwrap()
            .contains("first-run")
    );
    // The folder-trust screen is answered by the collector, not reported as a blocker.
    assert!(quota::claude_blocker("Permission Required: trust").is_none());
    assert!(quota::claude_trust_prompt(
        "Do you trust the files in this folder?"
    ));
    assert!(quota::claude_blocker("Claude Code v2\n$\nCurrent session\n27% used").is_none());
}
