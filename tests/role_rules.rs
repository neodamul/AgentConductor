#![cfg(unix)]
//! R-47 / DD-55 / QAS-40: operator role rules constrain the planning coordinator.
use agentconductor::ui::setup::Startup;
use std::{path::Path, process::Command};
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

fn plan_cli(
    root: &Path,
    mode: &str,
    roles: Option<serde_json::Value>,
    name: &str,
) -> std::process::Output {
    let (_, plan) = (0, support::setup("shared").1);
    let mut planner = plan.agents[0].clone();
    planner.id = "planner".into();
    planner.args = vec![mode.into()];
    planner.permission_mode = "read_only".into();
    let profiles = root.join(format!("{name}-profiles.json"));
    std::fs::write(
        &profiles,
        serde_json::to_vec(&vec![plan.agents[0].clone(), planner]).unwrap(),
    )
    .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_agentconductor"));
    command
        .arg("plan")
        .arg(&profiles)
        .arg("planner")
        .arg(root.join("repo"))
        .arg(root.join(name))
        .arg("Add a feature");
    if let Some(roles) = roles {
        let path = root.join(format!("{name}-roles.json"));
        std::fs::write(&path, roles.to_string()).unwrap();
        command.arg(path);
    }
    command.output().unwrap()
}

#[test]
fn planner_receives_rules_and_rejects_plans_that_break_them() {
    let (root, _) = support::setup("shared");
    let ok = plan_cli(
        &root,
        "plan-roles",
        Some(
            serde_json::json!({"development":["mock"],"testing":["mock"],"documentation":["mock"]}),
        ),
        "ok",
    );
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let prompt = std::fs::read_to_string(root.join("ok/prompt.txt")).unwrap();
    assert!(
        prompt.contains("Role rules") && prompt.contains("development: mock"),
        "{prompt}"
    );
    let plan: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("ok/plan.json")).unwrap()).unwrap();
    assert_eq!(plan["tasks"][0]["role"], "development");

    let wrong = plan_cli(
        &root,
        "plan-roles",
        Some(serde_json::json!({"development":["planner"]})),
        "wrong",
    );
    assert!(!wrong.status.success());
    let error = String::from_utf8_lossy(&wrong.stderr);
    assert!(
        error.contains("task-0") && error.contains("development"),
        "{error}"
    );

    let missing = plan_cli(
        &root,
        "plan",
        Some(serde_json::json!({"testing":["mock"]})),
        "missing",
    );
    assert!(
        String::from_utf8_lossy(&missing.stderr).contains("role"),
        "tasks must declare a role"
    );
    let legacy = plan_cli(&root, "plan", None, "legacy");
    assert!(legacy.status.success(), "no rules behaves as before");
    let legacy_plan = std::fs::read_to_string(root.join("legacy/plan.json")).unwrap();
    assert!(
        !legacy_plan.contains("\"role\""),
        "earlier plans keep their shape and digest"
    );
}

#[test]
fn workspace_saves_valid_rules_and_passes_them_to_planning() {
    let (root, plan) = support::setup("shared");
    let mut profiles = plan.agents;
    let mut planner = profiles[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan-roles".into()];
    planner.permission_mode = "read_only".into();
    profiles.push(planner);
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let save = |roles: serde_json::Value| {
        let revision = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["revision"].clone();
        let config = serde_json::json!({"profiles":profiles,"repository":root.join("repo"),"goal":"Feature","coordinator":"planner","workspace_mode":"shared","roles":roles});
        startup.dispatch(
            serde_json::json!({"kind":"save","revision":revision,"config":config})
                .to_string()
                .as_bytes(),
        )
    };
    assert!(
        save(serde_json::json!({"marketing":["mock"]})).is_err(),
        "unknown role"
    );
    assert!(
        save(serde_json::json!({"testing":["nobody"]})).is_err(),
        "unregistered agent"
    );
    save(serde_json::json!({"development":["mock"],"testing":["mock"],"documentation":["mock"],"verification":["planner"]})).unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(snapshot["config"]["roles"]["testing"][0], "mock");
    let revision = snapshot["revision"].clone();
    startup
        .dispatch(
            serde_json::json!({"kind":"prepare","revision":revision})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let ready = loop {
        let s = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if s["job"]["state"] != "running" {
            break s;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    assert_eq!(ready["proposal"]["phase"], "ready", "{}", ready["job"]);
    assert_eq!(ready["proposal"]["plan"]["tasks"][1]["role"], "testing");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn roles_table_sits_in_workspace_settings_and_tasks_show_their_role() {
    let settings = &HTML
        [HTML.find("Workspace settings").unwrap()..HTML.find("class=\"startup-actions\"").unwrap()];
    assert!(
        settings.contains("id=\"start-roles\""),
        "role table before Generate plan"
    );
    assert!(
        SCRIPT.contains("draft.roles"),
        "roles are saved with the workspace"
    );
    assert!(
        SCRIPT.contains("task.role"),
        "plan review shows each task's role"
    );
    let one = HTML.find("1. Workspace settings").expect("step 1");
    let two = HTML.find("2. Agent roles").expect("step 2");
    let three = HTML.find("3. Review the plan").expect("step 3");
    assert!(one < two && two < three);
}
