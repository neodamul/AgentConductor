use agentconductor::{adapters, models, plan::AgentProfile, protocol::Provider, workflow};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;
fn profile(root: &std::path::Path, mode: &str) -> AgentProfile {
    let executable = root.join(if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    });
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/claude_policy_agent.rs", "-o"])
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    serde_json::from_value(json!({"id":"policy","parser":"claude","executable":executable,"args":adapters::full_access_args(Provider::Claude).unwrap(),"permission_mode":"full_access","environment":{"AC_POLICY_MODE":mode}})).unwrap()
}
#[tokio::test]
async fn actual_workers_block_policy_denials_preserve_partial_work_and_cancel_late_writes() {
    for workspace in ["shared", "worktree"] {
        let (root, mut plan) = support::setup(workspace);
        let mut agent = profile(&root, "default");
        plan.tasks.truncate(1);
        plan.tasks[0].agent = agent.id.clone();
        plan.tasks[0].test_command = vec![agent.executable.clone(), "test".into()];
        for mode in [
            "default",
            "denied",
            "missing",
            "partial-denied",
            "delayed-denied",
            "quota-noise-denied",
            "bypassPermissions",
        ] {
            let marker = root.join(format!("prompt-{mode}"));
            agent.environment.insert(
                "AC_POLICY_USER_MARKER".into(),
                marker.to_str().unwrap().into(),
            );
            agent
                .environment
                .insert("AC_POLICY_MODE".into(), mode.into());
            plan.agents = vec![agent.clone()];
            let run = root.join(format!("run-{mode}"));
            let report = workflow::run(plan.clone(), &root.join("repo"), &run, 5)
                .await
                .unwrap();
            if mode == "bypassPermissions" {
                assert_eq!(
                    report.state, "waiting_approval",
                    "{workspace}/{mode}: {:?}",
                    report.error
                );
            } else {
                assert_eq!(
                    report.state, "blocked",
                    "{workspace}/{mode}: {:?}",
                    report.error
                );
                assert_eq!(report.tasks[0].status, "permission_denied");
                assert_eq!(report.tasks[0].attempt, 1);
                assert!(report.candidate.is_none());
                assert!(report.tests.is_empty());
                assert!(!workflow::repairable(&report));
            }
            if mode == "default" {
                assert!(
                    !marker.exists(),
                    "mismatched permission must not consume a user generation request"
                );
                tokio::time::sleep(std::time::Duration::from_millis(1600)).await;
                assert!(!report.tasks[0].cwd.join("late-write.txt").exists());
            }
            if mode == "partial-denied" {
                assert_eq!(
                    std::fs::read_to_string(report.tasks[0].cwd.join("one.txt")).unwrap(),
                    "partial\n"
                );
            }
            assert!(workflow::merge(&run).is_err());
            workflow::release_run_worktrees(&run);
            if workspace == "shared" {
                let _ = std::fs::remove_file(root.join("repo/one.txt"));
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[tokio::test]
async fn metadata_reports_actual_mode_without_sending_a_generation_prompt() {
    let (root, _) = support::setup("shared");
    let mut p = profile(&root, "default");
    let capture = root.join("argv.txt");
    p.environment
        .insert("AC_POLICY_CAPTURE".into(), capture.to_str().unwrap().into());
    let dir = root.join("metadata");
    let catalog = models::discover(&p, &dir).await.unwrap();
    assert_eq!(catalog.models, ["fixture-model"]);
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("permission.json")).unwrap()).unwrap();
    assert_eq!(evidence["effective"], "default");
    assert_eq!(evidence["requested"], "full_access");
    let captured = std::fs::read_to_string(capture).unwrap();
    for flag in adapters::full_access_args(Provider::Claude)
        .unwrap()
        .iter()
        .skip(
            adapters::invocation(Provider::Claude, None, "unused")
                .unwrap()
                .args
                .len(),
        )
    {
        assert!(captured.lines().any(|s| s == flag));
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn permission_denial_waits_for_human_reassignment_and_preserves_partial_files() {
    for mode in ["shared", "worktree"] {
        let (root, mut plan) = support::setup(mode);
        let mut backup = plan.agents[0].clone();
        backup.id = "backup".into();
        backup.args = vec!["task-preserve".into()];
        backup.quota_group = Some("backup".into());
        let mut paid = profile(&root, "partial-denied");
        paid.fallback = Some(agentconductor::plan::Fallback {
            agent: "backup".into(),
            model: None,
        });
        plan.tasks.truncate(1);
        plan.tasks[0].agent = paid.id.clone();
        plan.tasks[0].test_command = vec![paid.executable.clone(), "test".into()];
        plan.agents = vec![paid, backup];
        let run = root.join("run");
        let blocked = workflow::run(plan, &root.join("repo"), &run, 5)
            .await
            .unwrap();
        assert_eq!(blocked.state, "blocked");
        assert_eq!(blocked.plan.tasks[0].agent, "policy");
        assert!(blocked.account_restrictions.is_empty());
        workflow::reassign(&run, "one.txt", "backup", None).unwrap();
        let resumed = workflow::resume_due(
            &run,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            5,
        )
        .await
        .unwrap();
        assert_eq!(resumed.state, "waiting_approval", "{:?}", resumed.error);
        assert_eq!(resumed.tasks[0].attempt, 2);
        assert_eq!(resumed.plan.tasks[0].agent, "backup");
        assert_eq!(
            std::fs::read_to_string(resumed.tasks[0].cwd.join("one.txt")).unwrap(),
            "partial\nimplemented\n"
        );
        assert!(workflow::merge(&run).is_err());
        workflow::release_run_worktrees(&run);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn asynchronous_registration_verification_surfaces_policy_mismatch_without_blocking_save() {
    use agentconductor::ui::setup::Startup;
    let (root, _) = support::setup("shared");
    let p = profile(&root, "default");
    let startup = Startup::open(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let saved=startup.dispatch(json!({"kind":"save_profiles","revision":state["revision"],"profiles":[p],"renames":{}}).to_string().as_bytes()).unwrap();
    assert_eq!(
        saved["config"]["profiles"][0]["permission_mode"],
        "full_access"
    );
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    startup
        .dispatch(
            json!({"kind":"discover","revision":state["revision"],"agent":"policy"})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let end = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if snapshot["config_hints"]["policy"] == "startup.claude_permission_mismatch" {
            break;
        }
        assert!(
            std::time::Instant::now() < end,
            "{:?}",
            snapshot["metadata_job"]
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
