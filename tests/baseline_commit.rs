#![cfg(unix)]
//! R-42 / DD-50 / QAS-34: an unclean repository is explained and committed only on choice.
use agentconductor::{git::Repository, ui::setup::Startup};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

/// Text the dialog shows: literal markup or an English catalog message bound by data-i18n.
fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(startup: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}

#[test]
fn unclean_repository_is_listed_and_committed_only_when_chosen() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":repo,"goal":"Add the export feature\nwith details","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    std::fs::write(repo.join("base.txt"), "work in progress\n").unwrap();
    std::fs::write(repo.join("new.txt"), "untracked\n").unwrap();
    let git = Repository::open(&repo).unwrap();
    let before = git.head().unwrap();

    let status = startup.dispatch(br#"{"kind":"baseline_status"}"#).unwrap();
    assert_eq!(status["clean"], false);
    let changes = status["changes"].to_string();
    assert!(
        changes.contains("base.txt") && changes.contains("new.txt"),
        "{changes}"
    );
    // Stop: nothing changes, planning stays refused.
    assert!(
        mutate(&startup, serde_json::json!({"kind":"draft"}))
            .unwrap_err()
            .contains("clean")
    );
    assert_eq!(git.head().unwrap(), before);

    assert!(
        startup
            .dispatch(br#"{"kind":"commit_baseline","revision":"stale"}"#)
            .is_err()
    );
    assert_eq!(git.head().unwrap(), before);
    let committed = mutate(&startup, serde_json::json!({"kind":"commit_baseline"})).unwrap();
    let head = git.head().unwrap();
    assert_ne!(head, before);
    assert_eq!(committed["head"], head.as_str());
    git.clean().unwrap();
    let message = String::from_utf8(git.command(&["log", "-1", "--format=%s"]).unwrap()).unwrap();
    assert_eq!(
        message.trim(),
        "AgentConductor baseline: Add the export feature"
    );
    let files =
        String::from_utf8(git.command(&["show", "--name-only", "--format="]).unwrap()).unwrap();
    assert!(files.contains("base.txt") && files.contains("new.txt"));
    assert_eq!(
        startup.dispatch(br#"{"kind":"baseline_status"}"#).unwrap()["clean"],
        true
    );
    assert_eq!(
        mutate(&startup, serde_json::json!({"kind":"commit_baseline"})).unwrap()["committed"],
        false
    );

    mutate(&startup, serde_json::json!({"kind":"draft"})).unwrap();
    assert_eq!(
        snapshot(&startup)["proposal"]["repository_base"],
        head.as_str()
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn os_metadata_files_are_ignored_locally_without_a_commit() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo");
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&startup, serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":repo,"goal":"Feature","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join(".DS_Store"), "finder").unwrap();
    std::fs::write(repo.join("src/.DS_Store"), "finder").unwrap();
    std::fs::write(repo.join("._notes"), "appledouble").unwrap();
    std::fs::write(repo.join("real.txt"), "work").unwrap();
    let git = Repository::open(&repo).unwrap();
    let before = git.head().unwrap();

    let status = startup.dispatch(br#"{"kind":"baseline_status"}"#).unwrap();
    assert_eq!(status["os_files"].as_array().unwrap().len(), 3, "{status}");
    assert!(
        startup
            .dispatch(br#"{"kind":"ignore_os_files","revision":"stale"}"#)
            .is_err()
    );
    let ignored = mutate(&startup, serde_json::json!({"kind":"ignore_os_files"})).unwrap();
    assert_eq!(
        ignored["status"]["clean"], false,
        "real.txt still needs a decision"
    );
    assert_eq!(ignored["status"]["changes"].as_array().unwrap().len(), 1);
    assert!(ignored["status"]["os_files"].as_array().unwrap().is_empty());
    let exclude = String::from_utf8(
        git.command(&["rev-parse", "--git-path", "info/exclude"])
            .unwrap(),
    )
    .unwrap();
    let exclude = repo.join(exclude.trim());
    let text = std::fs::read_to_string(&exclude).unwrap();
    assert_eq!(text.lines().filter(|l| *l == ".DS_Store").count(), 1);
    assert_eq!(text.lines().filter(|l| *l == "._*").count(), 1);
    // Idempotent: a second call adds nothing.
    mutate(&startup, serde_json::json!({"kind":"ignore_os_files"})).unwrap();
    assert_eq!(std::fs::read_to_string(&exclude).unwrap(), text);
    assert_eq!(git.head().unwrap(), before, "ignoring never commits");
    std::fs::remove_file(repo.join("real.txt")).unwrap();
    assert_eq!(
        startup.dispatch(br#"{"kind":"baseline_status"}"#).unwrap()["clean"],
        true
    );
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
