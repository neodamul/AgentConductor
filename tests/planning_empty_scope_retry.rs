//! GitHub #251: a phased plan with an empty task scope gets one focused correction.
use agentconductor::{plan::AgentProfile, planner, roles::RoleRules};
use std::{fs, process::Command};

#[tokio::test]
async fn retries_once_when_a_verification_task_has_no_paths() {
    let root = std::env::temp_dir().join(format!(
        "ac-empty-scope-repair-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let repo = root.join("repo");
    fs::create_dir_all(&repo).unwrap();
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
    let executable = root.join("planner");
    assert!(
        Command::new("rustc")
            .args(["tests/support/overlap_repair_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let marker = root.join("attempt");
    let profile: AgentProfile = serde_json::from_value(serde_json::json!({
        "id":"claude", "parser":"claude", "executable":executable,
        "args":["fixture"], "permission_mode":"read_only",
        "environment":{
            "AC_PLAN_REPAIR_MARKER":marker.to_str().unwrap(),
            "AC_PLAN_FIRST_A":"[]", "AC_PLAN_FIRST_B":"[\"rules.rs\"]",
            "AC_PLAN_EXPECT":"paths is empty"
        }
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
        "Verify the ladder game",
        &[profile],
        "claude",
        &repo,
        &root.join("planning"),
        &inputs,
    )
    .await
    .unwrap();
    assert_eq!(result.tasks[0].paths, ["selection.rs"]);
    assert_eq!(fs::read_to_string(&marker).unwrap(), "2");
    assert!(slug.join("plan/plan.json").exists());
    fs::remove_dir_all(root).unwrap();
}
