#![cfg(unix)]
//! GitHub main issue #234: a periodic snapshot with a proposal must not start Git.
use agentconductor::ui::setup::Startup;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[path = "support/env.rs"]
mod env;
#[path = "support/workflow.rs"]
mod support;

struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn snapshot_with_a_proposal_succeeds_when_git_is_not_reachable() {
    let (root, plan) = support::setup("shared");
    let _cleanup = Cleanup(root.clone());
    let executable = root.join("blocked-planner");
    fs::write(&executable, "#!/bin/sh\nsleep 20\n").unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();

    let mut profile = plan.agents[0].clone();
    profile.executable = executable.to_string_lossy().into_owned();
    profile.args.clear();
    profile.permission_mode = "read_only".into();
    let startup = Startup::open(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = |startup: &Startup| startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let before = snapshot(&startup);
    let config = serde_json::json!({"profiles":[profile],"repository":root.join("repo"),"goal":"Blocked plan","coordinator":"mock","workspace_mode":"shared"});
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":before["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let saved = snapshot(&startup);
    startup
        .dispatch(
            serde_json::json!({"kind":"prepare","revision":saved["revision"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();

    let state = snapshot(&startup);
    let proposal = PathBuf::from(state["proposal"]["directory"].as_str().unwrap());
    let deadline = Instant::now() + Duration::from_secs(10);
    while !proposal.join("process/owner.json").exists() {
        assert!(Instant::now() < deadline, "planner process did not start");
        std::thread::sleep(Duration::from_millis(20));
    }

    let with_git = snapshot(&startup)["revision"].clone();
    let mut environment = env::lock();
    environment.set("PATH", "");
    let without_git = startup.dispatch(br#"{"kind":"snapshot"}"#);
    drop(environment);
    startup.dispatch(br#"{"kind":"stop_planning"}"#).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while snapshot(&startup)["job"]["state"] == "running" {
        assert!(Instant::now() < deadline, "planner process did not stop");
        std::thread::sleep(Duration::from_millis(20));
    }

    assert!(
        without_git.is_ok(),
        "snapshot unexpectedly needs Git: {without_git:?}"
    );
    assert_eq!(without_git.unwrap()["revision"], with_git);
}
