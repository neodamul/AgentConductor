use agentconductor::ui::setup::Startup;
use serde_json::json;
use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

struct TempRoot(PathBuf);

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn dispatch_latency() -> Duration {
    let root = TempRoot(std::env::temp_dir().join(format!(
        "ac-chat-timeout-{}",
        agentconductor::native_guard::session_id().unwrap()
    )));
    let repo = root.0.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
    ] {
        assert!(
            Command::new("git")
                .current_dir(&repo)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::write(repo.join("base.txt"), "base\n").unwrap();
    assert!(
        Command::new("git")
            .current_dir(&repo)
            .args(["add", "."])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&repo)
            .args(["commit", "-m", "base"])
            .status()
            .unwrap()
            .success()
    );

    let startup =
        Startup::open(&root.0.join("desktop"), &std::env::current_exe().unwrap()).unwrap();
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let config = json!({
        "profiles":[{"id":"coordinator","executable":env!("CARGO_BIN_EXE_test_agent_fixture"),"args":["-p"],"parser":"claude","permission_mode":"read_only"}],
        "repository":repo,"goal":"answer questions","coordinator":"coordinator","workspace_mode":"shared"
    });
    startup
        .dispatch(
            json!({"kind":"save","revision":before["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let saved = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();

    let real_git = String::from_utf8(Command::new("which").arg("git").output().unwrap().stdout)
        .unwrap()
        .trim()
        .to_owned();
    let bin = root.0.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let slow_git = bin.join("git");
    std::fs::write(&slow_git, format!("#!/bin/sh\ncase \"$*\" in *'rev-parse HEAD'*) sleep 20;; esac\nexec \"{real_git}\" \"$@\"\n")).unwrap();
    std::fs::set_permissions(&slow_git, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut environment = super::test_env::lock();
    environment.set("PATH", format!("{}:/usr/bin:/bin", bin.display()));

    let started = Instant::now();
    let result = startup.dispatch(
        json!({
            "kind":"chat","revision":saved["chat_revision"],
            "text":"Is this responsive?"
        })
        .to_string()
        .as_bytes(),
    );
    assert!(result.is_ok(), "chat submission failed: {result:?}");
    let elapsed = started.elapsed();
    drop(startup);
    drop(environment);
    elapsed
}
