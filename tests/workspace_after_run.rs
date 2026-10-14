//! GitHub #4: a retained completed Agent Room must not block workspace creation.
use agentconductor::ui::Room;
use std::{fs, path::Path};
#[path = "support/workflow.rs"]
mod support;

#[test]
fn completed_room_allows_new_workspace_but_active_run_remains_protected() {
    let (root, plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let workspace = root.join("workspace");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    drop(Room::startup(&workspace, exe).unwrap());
    let run = workspace.join("run-completed");
    fs::create_dir(&run).unwrap();
    let mut report = serde_json::json!({"schema_version":1,"state":"completed","repo":root.join("repo"),"base":"base","branch":"main","plan":plan,"tasks":[],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")});
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "fixture report")
        .unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_slice(&fs::read(workspace.join("startup.json")).unwrap()).unwrap();
    saved["last_run"] = serde_json::json!(run);
    saved["config"]["auto_resume"] = serde_json::json!(false);
    fs::write(
        workspace.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let room = Room::startup(&workspace, exe).unwrap();
    room.dispatch(br#"{"kind":"open_run"}"#).unwrap();
    room.dispatch(br#"{"kind":"home"}"#).unwrap();
    report["state"] = serde_json::json!("running");
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "fixture report")
        .unwrap();
    assert!(
        room.dispatch(br#"{"kind":"create_workspace","name":"Next project"}"#)
            .is_err(),
        "an active execution must not be orphaned"
    );
    assert_eq!(
        room.dispatch(br#"{"kind":"open_run"}"#).unwrap()["run"],
        serde_json::json!(run.canonicalize().unwrap()),
        "failed navigation keeps the previous room"
    );
    report["state"] = serde_json::json!("completed");
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "fixture report")
        .unwrap();
    let created = room
        .dispatch(br#"{"kind":"create_workspace","name":"Next project"}"#)
        .unwrap();
    assert_ne!(
        created["current"],
        serde_json::json!(workspace.canonicalize().unwrap())
    );
    assert_eq!(
        room.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["config"]["name"],
        "Next project"
    );
    assert!(
        room.dispatch(br#"{"kind":"snapshot","room":true}"#)
            .is_err(),
        "old controller detached after successful navigation"
    );
    assert!(
        run.join("report.json").exists(),
        "completed evidence is retained"
    );
    drop(room);
    fs::remove_dir_all(root).unwrap();
}
