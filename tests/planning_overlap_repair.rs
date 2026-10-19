//! A malformed overlapping phase is fed back to the coordinator once.
use agentconductor::{plan::AgentProfile, planner, roles::RoleRules};
use std::{fs, process::Command};

#[tokio::test]
async fn retries_phase_scopes_once_after_overlap_validation() {
    let root = std::env::temp_dir().join(format!(
        "ac-overlap-repair-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(root.join("repo")).unwrap();
    let repo = root.join("repo");
    for args in [
        vec!["init"],
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["add", "."],
        vec!["commit", "--allow-empty", "-m", "base"],
    ] {
        assert!(
            Command::new("git")
                .args(args)
                .current_dir(&repo)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let exe = root.join("planner");
    assert!(
        Command::new("rustc")
            .args(["tests/support/overlap_repair_agent.rs", "-o"])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let profile: AgentProfile = serde_json::from_value(serde_json::json!({
        "id":"claude", "parser":"claude", "executable":exe,
        "args":["fixture"], "permission_mode":"read_only",
        "environment":{"AC_PLAN_REPAIR_MARKER":root.join("attempt").to_str().unwrap()}
    }))
    .unwrap();
    let slug = root.join("slug");
    let inputs = planner::PlanInputs {
        roles: &RoleRules::new(),
        reference: None,
        context: None,
        rules: None,
        phases_root: Some(&slug),
    };
    let result = planner::generate_from(
        "Create a ladder game",
        &[profile],
        "claude",
        &repo,
        &root.join("planning"),
        &inputs,
    )
    .await
    .unwrap();
    assert_eq!(result.tasks[0].paths, ["selection.rs"]);
    assert!(
        fs::read_to_string(root.join("attempt"))
            .unwrap()
            .contains("2")
    );
    assert!(slug.join("plan/plan.json").exists());
    fs::remove_dir_all(root).unwrap();
}
