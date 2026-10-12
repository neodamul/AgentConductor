use agentconductor::ui::Room;
use serde_json::{Value, json};
fn call(room: &Room, value: Value) -> Result<Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
fn save(room: &Room, folder: &std::path::Path) -> Result<Value, String> {
    let snapshot = call(room, json!({"kind":"snapshot"}))?;
    call(
        room,
        json!({"kind":"save","revision":snapshot["revision"],"config":{
            "profiles":[{"id":"mock","parser":"custom","executable":env!("CARGO_BIN_EXE_agentconductor"),"args":["{prompt}"],"permission_mode":"read_only"}],
            "coordinator":"mock","repository":folder,"goal":"Build a new project","name":"New project","workspace_mode":"shared"
        }}),
    )
}
#[test]
fn a_new_project_folder_saves_without_git_and_planning_has_no_side_effects() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    let project = root.join("new project");
    std::fs::create_dir_all(&project).unwrap();
    let workspace = root.join("workspace");
    let room = Room::startup(
        &workspace,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    save(&room, &project).expect("saving settings does not require a Git repository");
    for kind in ["prepare", "draft", "baseline_status"] {
        let snapshot = call(&room, json!({"kind":"snapshot"})).unwrap();
        let packet = if kind == "baseline_status" {
            json!({"kind":kind})
        } else {
            json!({"kind":kind,"revision":snapshot["revision"]})
        };
        let error = call(&room, packet).unwrap_err();
        assert!(error.contains("Git repository"), "{kind}: {error}");
        assert!(!error.contains("rev-parse"), "{error}");
    }
    assert!(!project.join(".git").exists());
    assert!(!project.join(".agentconductor").exists());
    let before = call(&room, json!({"kind":"snapshot"})).unwrap();
    assert!(before["config"]["slugs"].as_array().unwrap().is_empty());
    let invalid = root.join("not-a-directory");
    std::fs::write(&invalid, b"file").unwrap();
    assert!(save(&room, &invalid).unwrap_err().contains("directory"));
    assert_eq!(
        call(&room, json!({"kind":"snapshot"})).unwrap()["config"],
        before["config"]
    );
    drop(room);
    let room = Room::startup(
        &workspace,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    assert_eq!(
        call(&room, json!({"kind":"snapshot"})).unwrap()["config"]["repository"],
        serde_json::to_value(project.canonicalize().unwrap()).unwrap()
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn an_unborn_repository_saves_but_requires_an_initial_commit_before_planning() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    let project = root.join("project");
    std::fs::create_dir_all(&project).unwrap();
    assert!(
        agentconductor::subprocess::command("git")
            .arg("init")
            .arg(&project)
            .output()
            .unwrap()
            .status
            .success()
    );
    let room = Room::startup(
        &root.join("workspace"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    save(&room, &project).unwrap();
    let snapshot = call(&room, json!({"kind":"snapshot"})).unwrap();
    let error = call(
        &room,
        json!({"kind":"prepare","revision":snapshot["revision"]}),
    )
    .unwrap_err();
    assert!(error.contains("initial commit"), "{error}");
    assert!(!project.join(".agentconductor").exists());
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn quota_refresh_does_not_require_git_or_an_execution_run() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    let project = root.join("plain project");
    std::fs::create_dir_all(&project).unwrap();
    let executable = root.join(if cfg!(windows) { "quota.exe" } else { "quota" });
    assert!(
        agentconductor::subprocess::command("rustc")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/quota_agent.rs"
            ))
            .arg("-o")
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let room = Room::startup(
        &root.join("workspace"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = call(&room, json!({"kind":"snapshot"})).unwrap();
    call(&room,json!({"kind":"save","revision":state["revision"],"config":{"profiles":[{"id":"codex","parser":"codex","executable":executable,"args":[]}],"coordinator":"codex","repository":project,"goal":"New project","workspace_mode":"shared"}})).unwrap();
    let state = call(&room, json!({"kind":"snapshot"})).unwrap();
    call(
        &room,
        json!({"kind":"quota","agent":"codex","revision":state["revision"]}),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let state = call(&room, json!({"kind":"snapshot"})).unwrap();
        if state["quota_job"]["state"] != "running" {
            assert_eq!(
                state["quotas"]["codex"]["windows"][0]["remaining_percent"], 73.0,
                "{state}"
            );
            assert_eq!(state["run_available"], false);
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!project.join(".git").exists());
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
