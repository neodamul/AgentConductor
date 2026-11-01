//! Git HEAD resolution stays off request paths for common and unusual repository layouts.
use agentconductor::git_head::{self, Head, Heads};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

#[path = "support/env.rs"]
mod env;

struct Temp(PathBuf);

impl Temp {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ac-git-head-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

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

fn repository(name: &str) -> Temp {
    let repo = Temp::new(name);
    git(&repo.0, &["init", "-q"]);
    std::fs::write(repo.0.join("a.txt"), "a").unwrap();
    git(&repo.0, &["add", "."]);
    git(&repo.0, &["commit", "-q", "-m", "one"]);
    repo
}

#[test]
fn loose_packed_and_detached_heads_are_read_without_git() {
    let mut environment = env::lock();
    let repo = repository("layouts");
    let path = std::env::var_os("PATH").unwrap_or_default();
    let real = git(&repo.0, &["rev-parse", "HEAD"]);
    environment.set("PATH", "");
    assert_eq!(git_head::head(&repo.0).unwrap(), real);
    environment.set("PATH", &path);
    git(&repo.0, &["pack-refs", "--all", "--prune"]);
    environment.set("PATH", "");
    assert_eq!(git_head::head(&repo.0).unwrap(), real);
    environment.set("PATH", &path);
    git(&repo.0, &["checkout", "-q", "--detach"]);
    environment.set("PATH", "");
    assert_eq!(git_head::head(&repo.0).unwrap(), real);
}

#[test]
fn a_new_commit_changes_the_head_immediately() {
    let _environment = env::lock();
    let repo = repository("commit");
    let before = git_head::head(&repo.0).unwrap();
    std::fs::write(repo.0.join("b.txt"), "b").unwrap();
    git(&repo.0, &["add", "."]);
    git(&repo.0, &["commit", "-q", "-m", "two"]);
    let after = git_head::head(&repo.0).unwrap();
    assert_ne!(before, after);
    assert_eq!(after, git(&repo.0, &["rev-parse", "HEAD"]));
}

#[test]
fn hostile_ref_names_are_rejected() {
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
fn linked_worktree_head_is_refreshed_in_the_background() {
    let _environment = env::lock();
    let repo = repository("worktree");
    let linked = Temp::new("linked");
    std::fs::remove_dir(&linked.0).unwrap();
    git(
        &repo.0,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "side",
            linked.0.to_str().unwrap(),
        ],
    );
    let real = git(&linked.0, &["rev-parse", "HEAD"]);
    assert_eq!(git_head::read(&linked.0), None);
    let heads = Heads::default();
    assert_eq!(heads.get(&linked.0), Head::Pending);
    let deadline = Instant::now() + Duration::from_secs(20);
    while heads.get(&linked.0) != Head::Known(real.clone()) {
        assert!(
            Instant::now() < deadline,
            "background refresh did not answer"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
