#![cfg(unix)]
//! GitHub #115: the workspace screen's chat request validates before launching any job.
use agentconductor::ui::Room;
use serde_json::json;
#[path = "support/env.rs"]
mod env;

#[test]
fn chat_requests_are_validated_and_the_snapshot_carries_a_chat_slot() {
    let root = std::env::temp_dir().join(format!("ac-startup-chat-{}", std::process::id()));
    // GitHub #222: the request diagnostics log lives under HOME; a test must not write to the real one.
    let mut env = env::lock();
    env.set("HOME", root.join("home"));
    let room = Room::startup(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert!(snapshot["chat"].is_null(), "no answer before any chat");
    let revision = snapshot["chat_revision"].clone();
    let send = |text: &str| {
        room.dispatch(
            json!({"kind":"chat","revision":revision,"text":text})
                .to_string()
                .as_bytes(),
        )
    };
    assert!(send("  ").is_err(), "empty question");
    assert!(send(&"x".repeat(9 * 1024)).is_err(), "over-long question");
    assert!(
        !send("How do I run it?").unwrap_err().is_empty(),
        "no coordinator is registered in a fresh workspace"
    );
    // GitHub #185: the conversation is stored with the workspace and can be cleared.
    let history = room.dispatch(br#"{"kind":"chat_history"}"#).unwrap();
    assert_eq!(
        history["turns"],
        json!([]),
        "nothing stored before a chat ran"
    );
    let reset = room.dispatch(br#"{"kind":"chat_reset"}"#).unwrap();
    assert_eq!(reset["turns"], json!([]));
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn chat_submission_does_not_wait_for_repository_head() {
    use std::{
        os::unix::fs::PermissionsExt,
        process::Command,
        time::{Duration, Instant},
    };

    let root = std::env::temp_dir().join(format!("ac-chat-timeout-{}", std::process::id()));
    let repo = root.join("repo");
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

    let mut environment = env::lock();
    environment.set("HOME", root.join("home"));
    let room = Room::startup(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let before = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let config = json!({
        "profiles":[{"id":"coordinator","executable":std::env::current_exe().unwrap(),"args":["-p"],"parser":"claude","permission_mode":"read_only"}],
        "repository":repo,"goal":"answer questions","coordinator":"coordinator","workspace_mode":"shared"
    });
    room.dispatch(
        json!({"kind":"save","revision":before["revision"],"config":config})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let saved = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    room.dispatch(
        json!({"kind":"draft","revision":saved["revision"]})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let reviewed = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();

    let git = Command::new("which").arg("git").output().unwrap();
    let real_git = String::from_utf8(git.stdout).unwrap().trim().to_owned();
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let slow_git = bin.join("git");
    std::fs::write(&slow_git, format!("#!/bin/sh\ncase \"$*\" in *'rev-parse HEAD'*) sleep 4;; esac\nexec \"{real_git}\" \"$@\"\n")).unwrap();
    std::fs::set_permissions(&slow_git, std::fs::Permissions::from_mode(0o755)).unwrap();
    environment.set("PATH", format!("{}:/usr/bin:/bin", bin.display()));

    let started = Instant::now();
    let result = room.dispatch(
        json!({"kind":"chat","revision":reviewed["chat_revision"],"text":"Is this responsive?"})
            .to_string()
            .as_bytes(),
    );
    assert!(result.is_ok(), "chat submission failed: {result:?}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "chat request waited for git HEAD: {:?}",
        started.elapsed()
    );
    drop(room);
    drop(environment);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn chat_rejects_a_stale_saved_context_without_a_git_head_lookup() {
    let root = std::env::temp_dir().join(format!("ac-chat-stale-context-{}", std::process::id()));
    let repository = root.join("repo");
    std::fs::create_dir_all(&repository).unwrap();
    let mut environment = env::lock();
    environment.set("HOME", root.join("home"));
    let room = Room::startup(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let initial = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let config = json!({"profiles":[],"coordinator":"","goal":"first goal","repository":repository,"workspace_mode":"shared"});
    room.dispatch(
        json!({"kind":"save","revision":initial["revision"],"config":config})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let reviewed = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let stale_chat_revision = reviewed["chat_revision"].clone();
    assert!(
        stale_chat_revision.is_string(),
        "chat context revision is exposed"
    );

    let changed = json!({"profiles":[],"coordinator":"","goal":"changed goal","repository":repository,"workspace_mode":"shared"});
    room.dispatch(
        json!({"kind":"save","revision":reviewed["revision"],"config":changed})
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    let result = room.dispatch(
        json!({"kind":"chat","revision":stale_chat_revision,"text":"hello"})
            .to_string()
            .as_bytes(),
    );
    let error = result.unwrap_err();
    assert!(
        error.contains("startup state changed since review"),
        "stale chat must be rejected before attempting coordinator launch: {error}"
    );
    drop(room);
    drop(environment);
    std::fs::remove_dir_all(root).unwrap();
}
