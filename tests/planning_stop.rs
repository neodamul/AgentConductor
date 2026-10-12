use agentconductor::ui::setup::Startup;
#[test]
fn stopping_idle_planning_is_an_explicit_noop() {
    let root = std::env::temp_dir().join(format!(
        "ac-stop-plan-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let result = startup.dispatch(br#"{"kind":"stop_planning"}"#).unwrap();
    assert_eq!(result["stopping"], false);
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
#[cfg(unix)]
#[test]
fn an_active_planner_stops_its_owned_process_and_releases_the_workspace() {
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    let (root, plan) = support::setup("shared");
    let executable = root.join("blocked-planner");
    fs::write(&executable, "#!/bin/sh\nsleep 20\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let mut profile = plan.agents[0].clone();
    profile.executable = executable.to_string_lossy().into();
    profile.args.clear();
    let workspace = root.join("desktop");
    let startup = Startup::open(
        &workspace,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let config = serde_json::json!({"profiles":[profile],"repository":root.join("repo"),"goal":"A blocked plan","coordinator":profile.id,"workspace_mode":"shared"});
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":before["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    startup
        .dispatch(
            serde_json::json!({"kind":"prepare","revision":before["revision"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let directory = std::path::PathBuf::from(state["proposal"]["directory"].as_str().unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !directory.join("process/owner.json").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    let stop = startup.dispatch(br#"{"kind":"stop_planning"}"#).unwrap();
    assert_eq!(stop["stopping"], true);
    while startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["job"]["state"] == "running" {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["job"]["state"],
        "stopped"
    );
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    startup.dispatch(serde_json::json!({"kind":"save_profiles","revision":state["revision"],"profiles":state["config"]["profiles"]}).to_string().as_bytes()).unwrap();
    let owner: agentconductor::process::Ownership =
        serde_json::from_slice(&fs::read(directory.join("process/owner.json")).unwrap()).unwrap();
    assert!(owner.stopped);
    owner.ensure_quiescent().unwrap();
    assert!(!directory.join("snapshot").exists());
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}
