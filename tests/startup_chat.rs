use agentconductor::ui::setup::Startup;
use serde_json::json;

#[cfg(unix)]
#[path = "support/env.rs"]
mod env;

#[test]
fn large_chat_request_uses_the_existing_startup_job_gate() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-startup-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(&root, &std::env::current_exe().unwrap()).unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let turns = (0..9)
        .map(|index| {
            json!({"role":if index == 8 {"operator"} else {"assistant"},"text":"x".repeat(8192)})
        })
        .collect::<Vec<_>>();
    let request = serde_json::to_vec(&json!({
        "kind":"chat", "revision":snapshot["revision"], "turns":turns
    }))
    .unwrap();
    assert!(request.len() > 64 * 1024 && request.len() < 128 * 1024);
    let error = startup.dispatch(&request).unwrap_err();
    assert!(
        error.contains("코디네이터") || error.contains("coordinator"),
        "{error}"
    );
    assert!(startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["job"].is_null());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn chat_submission_does_not_wait_for_repository_head() {
    use std::{
        os::unix::fs::PermissionsExt,
        process::Command,
        time::{Duration, Instant},
    };

    let root = std::env::temp_dir().join(format!(
        "ac-chat-timeout-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
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

    let startup = Startup::open(&root.join("desktop"), &std::env::current_exe().unwrap()).unwrap();
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
    startup
        .dispatch(
            json!({"kind":"draft","revision":saved["revision"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();

    let git = Command::new("which").arg("git").output().unwrap();
    let real_git = String::from_utf8(git.stdout).unwrap().trim().to_owned();
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let slow_git = bin.join("git");
    std::fs::write(
        &slow_git,
        format!(
            "#!/bin/sh\ncase \"$*\" in *'rev-parse HEAD'*) sleep 20;; esac\nexec \"{real_git}\" \"$@\"\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(&slow_git, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut environment = env::lock();
    environment.set("PATH", format!("{}:/usr/bin:/bin", bin.display()));

    let started = Instant::now();
    let result = startup.dispatch(
        json!({"kind":"chat","revision":saved["revision"],"turns":[{"role":"operator","text":"Is this responsive?"}]}).to_string().as_bytes(),
    );
    assert!(result.is_ok(), "chat submission failed: {result:?}");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "chat request waited for git HEAD: {:?}",
        started.elapsed()
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
