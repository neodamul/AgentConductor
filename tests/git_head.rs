//! GitHub #234 (DD-203): the repository HEAD is read from the Git directory without starting Git.
use agentconductor::git_head;
use std::{path::Path, process::Command};
#[path = "support/env.rs"]
mod env;

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args([
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=f@example.invalid",
        ])
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(output.status.success(), "{args:?}: {output:?}");
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn repository(name: &str) -> std::path::PathBuf {
    let repo = std::env::temp_dir().join(format!("ac-git-head-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&repo);
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    std::fs::write(repo.join("a.txt"), "a").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "one"]);
    repo
}

#[test]
fn the_file_read_head_equals_git_for_loose_packed_and_detached_heads() {
    let repo = repository("layouts");
    let mut env = env::lock();
    // With no `git` reachable, only a file read can answer.
    let real = git(&repo, &["rev-parse", "HEAD"]);
    env.set("PATH", "");
    assert_eq!(git_head::head(&repo).unwrap(), real, "loose ref");
    env.remove("PATH");
    git(&repo, &["pack-refs", "--all", "--prune"]);
    env.set("PATH", "");
    assert_eq!(git_head::head(&repo).unwrap(), real, "packed refs");
    env.remove("PATH");
    git(&repo, &["checkout", "-q", "--detach"]);
    env.set("PATH", "");
    assert_eq!(git_head::head(&repo).unwrap(), real, "detached HEAD");
    drop(env);
    std::fs::remove_dir_all(repo).unwrap();
}

#[test]
fn a_new_commit_is_seen_at_once() {
    let repo = repository("commit");
    let _env = env::lock();
    let before = git_head::head(&repo).unwrap();
    std::fs::write(repo.join("b.txt"), "b").unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-q", "-m", "two"]);
    let after = git_head::head(&repo).unwrap();
    assert_ne!(before, after);
    assert_eq!(after, git(&repo, &["rev-parse", "HEAD"]));
    std::fs::remove_dir_all(repo).unwrap();
}

#[test]
fn an_unusual_layout_is_answered_by_git_and_a_non_repository_still_fails() {
    // An unborn branch has no ref file: the old Git call answers (with its own error).
    let unborn = std::env::temp_dir().join(format!("ac-git-head-unborn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&unborn);
    std::fs::create_dir_all(&unborn).unwrap();
    git(&unborn, &["init", "-q"]);
    let _env = env::lock();
    assert!(git_head::head(&unborn).is_err());
    let plain = std::env::temp_dir().join(format!("ac-git-head-plain-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&plain);
    std::fs::create_dir_all(&plain).unwrap();
    assert!(git_head::head(&plain).is_err());
    std::fs::remove_dir_all(unborn).unwrap();
    std::fs::remove_dir_all(plain).unwrap();
}

#[test]
fn a_hostile_ref_name_never_leaves_the_refs_directory() {
    assert_eq!(
        git_head::resolve_ref_name("refs/heads/main"),
        Some("refs/heads/main")
    );
    for bad in [
        "../outside",
        "refs/../../x",
        "/etc/passwd",
        "refs\\heads\\x",
        "HEAD",
        "refs/heads/.invalid",
    ] {
        assert_eq!(git_head::resolve_ref_name(bad), None, "{bad}");
    }
}

#[test]
fn a_layout_the_files_cannot_answer_is_refreshed_in_the_background() {
    use agentconductor::git_head::{Head, Heads};
    use std::time::{Duration, Instant};
    let repo = repository("worktree");
    let linked = std::env::temp_dir().join(format!("ac-git-head-linked-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&linked);
    git(
        &repo,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "side",
            linked.to_str().unwrap(),
        ],
    );
    let _env = env::lock();
    let real = git(&linked, &["rev-parse", "HEAD"]);
    // `.git` is a file here, so only Git can say what HEAD is.
    assert_eq!(git_head::read(&linked), None);
    let heads = Heads::default();
    assert_eq!(
        heads.get(&linked),
        Head::Pending,
        "the request does not wait for Git"
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    while heads.get(&linked) != Head::Known(real.clone()) {
        assert!(
            Instant::now() < deadline,
            "the background refresh never answered"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    std::fs::remove_dir_all(&linked).unwrap();
    std::fs::remove_dir_all(repo).unwrap();
}
