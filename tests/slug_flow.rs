#![cfg(unix)]
//! R-55–R-59 / DD-61–DD-65 increment 6a: the workspace drives slugs, phased plans, phase
//! runs, the next phase and plan rules end to end.
use agentconductor::{slug_progress::progress, ui::setup::Startup, workflow};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

fn snapshot(s: &Startup) -> serde_json::Value {
    s.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(s: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(s)["revision"].clone();
    s.dispatch(packet.to_string().as_bytes())
}
fn settle(s: &Startup) -> serde_json::Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let state = snapshot(s);
        if state["job"]["state"] != "running" {
            return state;
        }
        assert!(std::time::Instant::now() < deadline, "job did not finish");
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

#[test]
fn goal_to_slug_phases_runs_and_rules() {
    let (root, plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let repo = root.join("repo").canonicalize().unwrap();
    let mut planner = plan.agents[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan-phases".into()];
    planner.permission_mode = "read_only".into();
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&startup, serde_json::json!({"kind":"save","config":{"name":"Health","profiles":[plan.agents[0].clone(), planner],"repository":repo,"goal":"Add the health check","coordinator":"planner","workspace_mode":"shared"}})).unwrap();

    // Plan rules: the workspace override reaches the coordinator.
    mutate(&startup, serde_json::json!({"kind":"save_plan_rules","scope":"workspace","text":"- Keep every phase small"})).unwrap();
    assert_eq!(
        snapshot(&startup)["config"]["plan_rules"],
        "- Keep every phase small"
    );

    mutate(
        &startup,
        serde_json::json!({"kind":"prepare","slug":"healthcheck_feature"}),
    )
    .unwrap();
    let ready = settle(&startup);
    assert_eq!(ready["proposal"]["phase"], "ready", "{}", ready["job"]);
    assert_eq!(ready["proposal"]["slug"], "healthcheck_feature");
    assert_eq!(ready["proposal"]["slug_phase"], "01-core");
    let ids: Vec<_> = ready["proposal"]["plan"]["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(ids, ["a", "b"]);
    let prompt = std::fs::read_to_string(
        Path::new(ready["proposal"]["directory"].as_str().unwrap()).join("prompt.txt"),
    )
    .unwrap();
    assert!(prompt.contains("Keep every phase small") && prompt.contains("\"phases\""));
    let tree = repo.join("healthcheck_feature");
    assert_eq!(ready["slugs"][0]["slug"], "healthcheck_feature");
    assert_eq!(ready["slugs"][0]["progress"]["current_phase"], "01-core");

    // Approve phase 1: tasks reach verification after tests, completed after merge.
    let accepted = mutate(
        &startup,
        serde_json::json!({"kind":"start","digest":ready["proposal"]["digest"]}),
    )
    .unwrap();
    settle(&startup);
    assert_eq!(progress(&tree)["phases"][0]["stage"], "verification");
    let run = Path::new(accepted["run"].as_str().unwrap());
    let report = workflow::load(run).unwrap();
    workflow::approve(run, &report.candidate.unwrap().commit).unwrap();
    workflow::merge(run).unwrap();
    assert_eq!(progress(&tree)["current_phase"], "02-docs");

    // The next phase comes from the slug tree, without another coordinator run.
    mutate(&startup, serde_json::json!({"kind":"next_phase"})).unwrap();
    let next = snapshot(&startup);
    assert!(
        next["job"].is_null() || next["job"]["kind"] != "plan",
        "no coordinator job"
    );
    assert_eq!(next["proposal"]["slug_phase"], "02-docs");
    assert_eq!(next["proposal"]["plan"]["tasks"][0]["id"], "c");

    // Generate plan for a goal that already has a plan reloads its current phase.
    let loaded = mutate(&startup, serde_json::json!({"kind":"prepare"})).unwrap();
    assert_eq!(loaded["phase"], "02-docs");
    let again = snapshot(&startup);
    assert!(
        again["job"].is_null() || again["job"]["kind"] != "plan",
        "no coordinator job"
    );
    assert_eq!(again["proposal"]["phase"], "ready");
    assert_eq!(again["proposal"]["slug_phase"], "02-docs");

    // Global rules can be edited too; the workspace override can be cleared.
    mutate(&startup, serde_json::json!({"kind":"save_plan_rules","scope":"global","text":"# Global\n- One rule"})).unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("home/.agentconductor/plan-rules.md")).unwrap(),
        "# Global\n- One rule"
    );
    mutate(
        &startup,
        serde_json::json!({"kind":"save_plan_rules","scope":"workspace","text":null}),
    )
    .unwrap();
    assert!(snapshot(&startup)["config"]["plan_rules"].is_null());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
