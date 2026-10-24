//! GitHub #6: one project workspace, independent goals and preserved completed evidence.
use agentconductor::{ui::Room, workflow};
use std::{fs, path::Path, process::Command};
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;
fn call(room: &Room, mut request: serde_json::Value) -> Result<serde_json::Value, String> {
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#)?;
    if !matches!(
        request["kind"].as_str(),
        Some("open_run" | "create_workspace" | "switch_workspace")
    ) {
        request["revision"] = snapshot["revision"].clone();
    }
    room.dispatch(request.to_string().as_bytes())
}
fn save(room: &Room, project: &Path, goal: &str) -> Result<serde_json::Value, String> {
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#)?;
    let mut config = snapshot["config"].clone();
    config["repository"] = serde_json::json!(project);
    config["goal"] = goal.into();
    call(room, serde_json::json!({"kind":"save","config":config}))
}
fn cli(args: &[&std::ffi::OsStr]) -> std::process::Output {
    let out = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    out
}
#[test]
fn completed_project_accepts_new_goals_restores_history_and_refuses_duplicate_binding() {
    let (root, mut plan) = support::setup("shared");
    plan.goal = "Build a ladder game".into();
    for task in &mut plan.tasks {
        task.id = task.id.trim_end_matches(".txt").into();
    }
    let home = root.join("home");
    let mut env = test_env::lock();
    env.set("HOME", &home);
    let workspace = home.join(".agentconductor/workspaces/project");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::startup(&workspace, exe).unwrap();
    save(&room, &root.join("repo"), &plan.goal).unwrap();
    call(
        &room,
        serde_json::json!({"kind":"create_slug","slug":"ladder_game"}),
    )
    .unwrap();
    let slug_root = root.join("repo/ladder_game");
    agentconductor::phases::write(
        &slug_root,
        "# Fixture analysis",
        &[agentconductor::phases::PhaseSpec {
            id: "01-core".into(),
            name: "Core".into(),
            tasks: plan.tasks.clone(),
        }],
    )
    .unwrap();
    let plan_file = workspace.join("fixture-plan.json");
    fs::write(&plan_file, serde_json::to_vec(&plan).unwrap()).unwrap();
    let validated: serde_json::Value =
        serde_json::from_slice(&cli(&["validate".as_ref(), plan_file.as_os_str()]).stdout).unwrap();
    let run = workspace.join("run-ladder");
    cli(&[
        "run".as_ref(),
        plan_file.as_os_str(),
        root.join("repo").as_os_str(),
        run.as_os_str(),
        validated["plan_digest"].as_str().unwrap().as_ref(),
        "--phase".as_ref(),
        slug_root.as_os_str(),
        "01-core".as_ref(),
    ]);
    let report = workflow::load(&run).unwrap();
    let candidate = report.candidate.unwrap().commit;
    cli(&["approve".as_ref(), run.as_os_str(), candidate.as_ref()]);
    cli(&["merge".as_ref(), run.as_os_str()]);
    drop(room);
    let mut saved: serde_json::Value =
        serde_json::from_slice(&fs::read(workspace.join("startup.json")).unwrap()).unwrap();
    saved["last_run"] = serde_json::json!(run);
    saved["config"]["auto_resume"] = false.into();
    fs::write(
        workspace.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let room = Room::startup(&workspace, exe).unwrap();
    call(&room, serde_json::json!({"kind":"open_run"})).unwrap();
    call(&room, serde_json::json!({"kind":"new_goal"})).unwrap();
    let first = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(first["config"]["goal"], "");
    assert!(first["config"]["active_slug"].is_null());
    assert_eq!(first["slugs"].as_array().unwrap().len(), 1);
    assert!(first["proposal"].is_null());
    assert_eq!(first["run_available"], false, "old run is not the new goal");
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"start","digest":validated["plan_digest"]})
        )
        .is_err(),
        "old approval cannot launch the new goal"
    );
    save(&room, &root.join("repo"), "Add a marble game").unwrap();
    call(
        &room,
        serde_json::json!({"kind":"create_slug","slug":"marble_game"}),
    )
    .unwrap();
    call(
        &room,
        serde_json::json!({"kind":"select_goal","slug":"ladder_game"}),
    )
    .unwrap();
    let old = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(old["config"]["goal"], plan.goal);
    assert_eq!(old["run_available"], true);
    assert_eq!(
        call(&room, serde_json::json!({"kind":"open_run"})).unwrap()["run"],
        serde_json::json!(run.canonicalize().unwrap())
    );
    let completed = workflow::load(&run).unwrap();
    for state in ["running", "waiting_approval", "waiting_availability"] {
        let mut active = completed.clone();
        active.state = state.into();
        let value = serde_json::to_value(&active).unwrap();
        agentconductor::state::Store::open(run.join("state.sqlite"))
            .unwrap()
            .record("workflow", &value, "controlled activity regression")
            .unwrap();
        fs::write(
            run.join("report.json"),
            serde_json::to_vec(&active).unwrap(),
        )
        .unwrap();
        assert!(
            call(&room, serde_json::json!({"kind":"new_goal"})).is_err(),
            "{state} must finish or stop before the same project changes goals"
        );
    }
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record(
            "workflow",
            &serde_json::to_value(&completed).unwrap(),
            "restored completed fixture",
        )
        .unwrap();
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&completed).unwrap(),
    )
    .unwrap();
    call(
        &room,
        serde_json::json!({"kind":"select_goal","slug":"marble_game"}),
    )
    .unwrap();
    let new = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(new["config"]["goal"], "Add a marble game");
    assert_eq!(new["slugs"].as_array().unwrap().len(), 2);
    assert_eq!(new["run_available"], false);
    assert!(new["proposal"].is_null());
    call(
        &room,
        serde_json::json!({"kind":"create_workspace","name":"Duplicate"}),
    )
    .unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let error = save(&room, &root.join("repo"), "Another goal").unwrap_err();
    assert!(error.contains("workspace"), "{error}");
    let after = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(
        after["config"], before["config"],
        "binding failure keeps the form state"
    );
    assert_eq!(workflow::load(&run).unwrap().state, "completed");
    assert!(run.join("report.json").exists());
    drop(room);
    fs::remove_dir_all(root).unwrap();
}
