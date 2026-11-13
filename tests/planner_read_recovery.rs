use agentconductor::{planner, protocol::Provider};
#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn actual_planner_accepts_snapshot_recovery_but_preserves_unresolved_failures() {
    for mode in ["recovered", "unresolved"] {
        let (root, plan) = support::setup("shared");
        let exe = root.join(if cfg!(windows) {
            "planner.exe"
        } else {
            "planner"
        });
        assert!(
            std::process::Command::new("rustc")
                .args(["tests/support/planner_read_recovery.rs", "-o"])
                .arg(&exe)
                .status()
                .unwrap()
                .success()
        );
        let mut profile = plan.agents[0].clone();
        profile.executable = exe.to_str().unwrap().into();
        profile.parser = Provider::Claude;
        profile.args.clear();
        profile.permission_mode = "read_only".into();
        profile
            .environment
            .insert("AC_PLANNING_RECOVERY".into(), mode.into());
        let result = planner::generate(
            "Create three files",
            &[profile],
            "mock",
            &root.join("repo"),
            &root.join("planning"),
        )
        .await;
        if mode == "recovered" {
            let plan = result.unwrap();
            assert_eq!(plan.tasks.len(), 3);
            plan.validate().unwrap();
        } else {
            assert!(result.unwrap_err().contains("permission_denials"));
            assert!(!root.join("planning/plan.json").exists());
        }
        assert!(!root.join("planning/snapshot").exists());
        agentconductor::git::Repository::open(&root.join("repo"))
            .unwrap()
            .clean()
            .unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
