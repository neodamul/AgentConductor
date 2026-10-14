use agentconductor::{
    cooperative::route::{self, Preference, Transport},
    planner::{self, PlanInputs},
    roles::RoleRules,
    workflow,
};
#[path = "support/workflow.rs"]
mod support;
#[tokio::test]
async fn local_selection_overrides_optional_tool_rules_in_the_actual_planning_prompt() {
    let (root, plan) = support::setup("shared");
    let mut profile = plan.agents[0].clone();
    profile.args = vec!["plan".into()];
    profile.permission_mode = "read_only".into();
    route::save_preference(
        &root,
        &Preference {
            transport: Transport::Local,
            port: 3111,
        },
    )
    .unwrap();
    let roles = RoleRules::new();
    let inputs = PlanInputs {
        roles: &roles,
        reference: None,
        context: None,
        rules: Some("Use agentmemory and optional open-source tools for communication"),
        phases_root: None,
    };
    planner::generate_from(
        "Verify files",
        &[profile.clone()],
        &profile.id,
        &root.join("repo"),
        &root.join("plan"),
        &inputs,
    )
    .await
    .unwrap();
    let prompt = std::fs::read_to_string(root.join("plan/prompt.txt")).unwrap();
    assert!(
        prompt.contains("Coordination transport: Local Rust"),
        "selected backend was not sent to the planner"
    );
    assert!(prompt.contains("not required") && prompt.contains("optional tool guidance"));
    std::fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn workers_receive_the_configured_backend_without_inventing_checkpoint_capability() {
    for (transport, expected) in [
        (Transport::Local, "Local Rust"),
        (Transport::Agentmemory, "agentmemory preferred"),
    ] {
        let (root, mut plan) = support::setup("shared");
        std::fs::write(root.join("startup.json"), "{}").unwrap();
        route::save_preference(
            &root,
            &Preference {
                transport,
                port: 3111,
            },
        )
        .unwrap();
        for task in &mut plan.tasks {
            task.prompt.push_str("\nSend progress using agentmemory.");
        }
        let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 10)
            .await
            .unwrap();
        assert_eq!(report.state, "waiting_approval");
        for task in &report.tasks {
            let prompt = std::fs::read_to_string(task.artifacts.join("prompt.txt")).unwrap();
            assert!(prompt.contains(&format!("Coordination transport: {expected}")));
            assert!(prompt.contains("normal CLI output"));
            assert!(!prompt.contains("checkpoint_v1):"));
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
#[test]
fn windows_picker_uses_the_system_dialog_without_starting_powershell() {
    let picker = include_str!("../src/ui/picker.rs");
    assert!(
        !picker.contains("powershell.exe"),
        "picker launches and compiles a helper for every click"
    );
}
