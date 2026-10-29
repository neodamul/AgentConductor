#![cfg(unix)]
//! GitHub #234 (DD-203): the periodic snapshot of a workspace with a proposal does not start Git.
use agentconductor::ui::setup::Startup;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};
#[path = "support/env.rs"]
mod env;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn a_snapshot_with_a_proposal_needs_no_git_process() {
    let (root, plan) = support::setup("shared");
    let executable = root.join("blocked-planner");
    fs::write(&executable, "#!/bin/sh\nsleep 20\n").unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    let mut profile = plan.agents[0].clone();
    profile.executable = executable.to_string_lossy().into();
    profile.args.clear();
    let startup = Startup::open(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = |startup: &Startup| startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let config = serde_json::json!({"profiles":[profile],"repository":root.join("repo"),"goal":"A blocked plan","coordinator":profile.id,"workspace_mode":"shared"});
    let before = snapshot(&startup);
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":before["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let before = snapshot(&startup);
    startup
        .dispatch(
            serde_json::json!({"kind":"prepare","revision":before["revision"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let state = snapshot(&startup);
    let directory = std::path::PathBuf::from(state["proposal"]["directory"].as_str().unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !directory.join("process/owner.json").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    {
        let mut env = env::lock();
        let with_git = snapshot(&startup)["revision"].clone();
        env.set("PATH", "");
        // A proposal makes the revision cover the repository HEAD; it must not need a `git` process.
        let without_git = startup
            .dispatch(br#"{"kind":"snapshot"}"#)
            .expect("snapshot without git");
        assert_eq!(
            without_git["revision"], with_git,
            "same bytes as before the change"
        );
    }
    startup.dispatch(br#"{"kind":"stop_planning"}"#).unwrap();
    while snapshot(&startup)["job"]["state"] == "running" {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}
