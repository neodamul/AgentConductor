use agentconductor::{planner::schema, protocol::Provider};
use serde_json::Value;
#[test]
fn phase_contract_requires_arrays_and_registered_agents() {
    let s = schema::for_plan(&["claude".into()], &[], true);
    assert_eq!(s["additionalProperties"], false);
    assert_eq!(s["properties"]["phases"]["type"], "array");
    let p = &s["properties"]["phases"]["items"];
    assert_eq!(p["properties"]["tasks"]["type"], "array");
    let t = &p["properties"]["tasks"]["items"];
    assert_eq!(
        t["properties"]["agent"]["enum"],
        serde_json::json!(["claude"])
    );
    for n in ["paths", "depends_on", "test_command"] {
        assert_eq!(t["properties"][n]["type"], "array");
    }
    assert_eq!(t["properties"]["execution_options"]["type"], "object");
    assert!(
        !t["required"]
            .as_array()
            .unwrap()
            .contains(&"execution_options".into())
    );
}
#[test]
fn native_cli_arguments_bind_the_exact_contract() {
    let s = schema::for_plan(&["claude".into()], &[], true);
    let path = std::path::Path::new("C:/project/output-schema.json");
    let mut claude = vec!["-p".into()];
    schema::bind(Provider::Claude, &mut claude, &s, path).unwrap();
    assert_eq!(claude[1], "--json-schema");
    assert_eq!(serde_json::from_str::<Value>(&claude[2]).unwrap(), s);
    let mut codex = vec!["exec".into()];
    schema::bind(Provider::Codex, &mut codex, &s, path).unwrap();
    assert_eq!(
        &codex[1..],
        &["--output-schema", "C:/project/output-schema.json"]
    );
    let mut custom = vec!["--user-flag".into()];
    schema::bind(Provider::Custom, &mut custom, &s, path).unwrap();
    assert_eq!(custom, vec!["--user-flag"]);
    assert!(schema::bind(Provider::Claude, &mut claude, &s, path).is_err());
}
#[test]
fn flat_plans_and_role_constraints_keep_their_contract() {
    let s = schema::for_plan(&["claude".into()], &["development".into()], false);
    assert!(s["properties"].get("phases").is_none());
    assert_eq!(s["properties"]["tasks"]["type"], "array");
    assert_eq!(
        s["properties"]["tasks"]["items"]["properties"]["role"]["enum"],
        serde_json::json!(["development"])
    );
}

#[tokio::test]
async fn actual_planner_requests_and_reads_schema_constrained_phases() {
    use agentconductor::{plan::AgentProfile, planner, roles::RoleRules};
    use std::{fs, process::Command};
    let root = std::env::temp_dir().join(format!(
        "ac-planning-schema-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(root.join("repo")).unwrap();
    let exe = root.join(if cfg!(windows) {
        "schema-fixture.exe"
    } else {
        "schema-fixture"
    });
    assert!(
        Command::new("rustc")
            .args(["tests/support/planning_schema_agent.rs", "-o"])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let repo = root.join("repo");
    fs::write(repo.join("one.txt"), "base").unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.name", "Fixture"],
        vec!["config", "user.email", "fixture@example.invalid"],
        vec!["add", "one.txt"],
        vec!["commit", "-m", "base"],
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
    let profile:AgentProfile=serde_json::from_value(serde_json::json!({"id":"claude","parser":"claude","executable":exe,"args":["fixture-plan"],"permission_mode":"read_only"})).unwrap();
    let roles = RoleRules::new();
    let phases = root.join("slug");
    let inputs = planner::PlanInputs {
        roles: &roles,
        reference: None,
        context: None,
        rules: None,
        phases_root: Some(&phases),
    };
    let plan = planner::generate_from(
        "Check schema",
        &[profile],
        "claude",
        &repo,
        &root.join("plan"),
        &inputs,
    )
    .await
    .unwrap();
    assert_eq!(plan.tasks[0].id, "a");
    let contract: Value =
        serde_json::from_slice(&fs::read(root.join("plan/output-schema.json")).unwrap()).unwrap();
    assert_eq!(contract["properties"]["phases"]["type"], "array");
    let stored: Value =
        serde_json::from_slice(&fs::read(phases.join("plan/plan.json")).unwrap()).unwrap();
    assert_eq!(stored["phases"][0]["id"], "01-core");
    fs::remove_dir_all(root).unwrap();
}

/// GitHub #251: the output contract asks for what `Plan::validate` demands.
#[test]
fn a_task_must_own_a_path_and_name_a_test_command() {
    for phased in [true, false] {
        let s = schema::for_plan(&["claude".into()], &[], phased);
        let tasks = if phased {
            &s["properties"]["phases"]["items"]["properties"]["tasks"]
        } else {
            &s["properties"]["tasks"]
        };
        for field in ["paths", "test_command"] {
            assert_eq!(
                tasks["items"]["properties"][field]["minItems"], 1,
                "{field}"
            );
        }
        assert_eq!(
            tasks["items"]["properties"]["depends_on"].get("minItems"),
            None
        );
    }
}
