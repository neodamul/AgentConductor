#![cfg(windows)]
use agentconductor::{executable, quota};
use serde_json::json;
#[test]
fn npm_windows_launcher_precedes_posix_shim() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("codex"), "#!/bin/sh").unwrap();
    std::fs::write(root.join("codex.cmd"), "@echo fixture").unwrap();
    assert_eq!(
        executable::resolve_in("codex", &[root.clone()]),
        Some(root.join("codex.cmd"))
    );
    assert_eq!(
        executable::resolve(root.join("codex").to_str().unwrap()),
        Some(root.join("codex.cmd"))
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn claude_conpty_reports_usage_without_generation_or_trust_approval() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let exe = root.join("terminal-fixture.exe");
    assert!(
        std::process::Command::new("rustc")
            .args([
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/tests/support/terminal_quota_agent.rs"
                ),
                "-o"
            ])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let mut profile:agentconductor::plan::AgentProfile=serde_json::from_value(json!({"id":"claude","parser":"claude","executable":exe,"args":[],"environment":{"AC_QUOTA_CASE":"normal"}})).unwrap();
    let usage = quota::query_claude(&profile, &root.join("normal"), &root)
        .await
        .unwrap();
    assert_eq!(usage.windows.len(), 2);
    assert_eq!(usage.windows[0].remaining_percent, Some(73.0));
    let owner: agentconductor::process::Ownership =
        serde_json::from_slice(&std::fs::read(root.join("normal/owner.json")).unwrap()).unwrap();
    owner.ensure_quiescent().unwrap();
    profile
        .environment
        .insert("AC_QUOTA_CASE".into(), "trust".into());
    assert!(
        quota::query_claude(&profile, &root.join("trust"), &root)
            .await
            .unwrap_err()
            .contains("trust")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn codex_quota_recovers_an_existing_extensionless_npm_profile() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let exe = root.join("quota-agent.exe");
    assert!(
        std::process::Command::new("rustc")
            .args([
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/quota_agent.rs"),
                "-o"
            ])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(root.join("codex"), "#!/bin/sh").unwrap();
    std::fs::write(
        root.join("codex.cmd"),
        format!("@echo off\r\n\"{}\" %*\r\n", exe.display()),
    )
    .unwrap();
    let p = serde_json::from_value(
        json!({"id":"codex","parser":"codex","executable":root.join("codex"),"args":[]}),
    )
    .unwrap();
    let usage = quota::query(&p, &root.join("query")).await.unwrap();
    assert_eq!(usage.windows[0].remaining_percent, Some(73.0));
    let owner: agentconductor::process::Ownership =
        serde_json::from_slice(&std::fs::read(root.join("query/owner.json")).unwrap()).unwrap();
    owner.ensure_quiescent().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
