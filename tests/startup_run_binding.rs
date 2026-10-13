use agentconductor::{folder, git::Repository, ui::Room, workflow};
use serde_json::{Value, json};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;
fn call(room: &Room, value: Value) -> Result<Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
#[tokio::test]
async fn an_existing_direct_folder_run_opens_but_unrelated_reports_are_refused() {
    let (root, plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    let workspace = root.join("desktop");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::startup(&workspace, exe).unwrap();
    let state = call(&room, json!({"kind":"snapshot"})).unwrap();
    call(&room,json!({"kind":"save","revision":state["revision"],"config":{"profiles":plan.agents,"coordinator":"mock","repository":project,"goal":plan.goal,"workspace_mode":"shared","auto_resume":false}})).unwrap();
    let state = call(&room, json!({"kind":"snapshot"})).unwrap();
    call(&room, json!({"kind":"draft","revision":state["revision"]})).unwrap();
    drop(room);
    let saved_path = workspace.join("startup.json");
    let mut saved: Value = serde_json::from_slice(&std::fs::read(&saved_path).unwrap()).unwrap();
    let planned = Path::new(saved["proposal"]["workspace"].as_str().unwrap()).to_owned();
    folder::activate(&Repository::open(&planned).unwrap(), &project).unwrap();
    let run = workspace.join("run-direct");
    let report = workflow::run(plan, &project, &run, 10).await.unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert_ne!(report.repo, planned);
    assert!(!project.join(".git").exists());
    saved["proposal"]["phase"] = json!("starting");
    saved["proposal"]["run"] = json!(run);
    saved["last_run"] = json!(run);
    std::fs::write(&saved_path, serde_json::to_vec(&saved).unwrap()).unwrap();
    let room = Room::startup(&workspace, exe).unwrap();
    call(&room, json!({"kind":"open_run"}))
        .expect("the already-started direct execution belongs to its reviewed planning copy");
    drop(room);
    let original = serde_json::to_value(&report).unwrap();
    for (field, value) in [
        ("repo", json!(planned)),
        ("base", json!("unrelated-base")),
        ("goal", json!("unrelated-goal")),
    ] {
        let mut changed = original.clone();
        if field == "goal" {
            changed["plan"][field] = value;
        } else {
            changed[field] = value;
        }
        // Legacy copy executions remain valid; use an unrelated project for the repo test.
        if field == "repo" {
            changed[field] = json!(root.canonicalize().unwrap());
        }
        agentconductor::state::Store::open(run.join("state.sqlite"))
            .unwrap()
            .record("workflow", &changed, "test mismatched run binding")
            .unwrap();
        std::fs::write(
            run.join("report.json"),
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        let room = Room::startup(&workspace, exe).unwrap();
        assert!(
            call(&room, json!({"kind":"open_run"})).is_err(),
            "mismatched {field} must be refused"
        );
        drop(room);
    }
    agentconductor::state::Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &original, "test restored run binding")
        .unwrap();
    std::fs::write(
        run.join("report.json"),
        serde_json::to_vec(&original).unwrap(),
    )
    .unwrap();
    std::fs::remove_dir_all(folder::history_directory(&project).unwrap()).unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
