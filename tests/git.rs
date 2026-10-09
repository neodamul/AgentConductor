use agentconductor::git::Repository;
use std::{path::PathBuf, process::Command};

fn repo() -> (PathBuf, Repository) {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root =
        std::env::temp_dir().join(format!("ac-git-{}-{stamp}-{sequence}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Test"],
        vec!["config", "user.email", "test@example.invalid"],
    ] {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    std::fs::write(root.join("base.txt"), "base\n").unwrap();
    for args in [vec!["add", "base.txt"], vec!["commit", "-m", "base"]] {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let repo = Repository::open(&root).unwrap();
    (root, repo)
}

#[test]
fn snapshot_includes_untracked_without_changing_head_or_real_index() {
    let (root, repo) = repo();
    let base = repo.head().unwrap();
    std::fs::write(root.join("new.txt"), "new\n").unwrap();
    let index = std::env::temp_dir().join(format!("ac-index-{}", base));
    let snapshot = repo.snapshot(&base, &["new.txt".into()], &index).unwrap();
    assert_eq!(snapshot.changed, ["new.txt"]);
    assert_eq!(repo.head().unwrap(), base);
    assert!(
        String::from_utf8(repo.command(&["diff", "--cached", "--name-only"]).unwrap())
            .unwrap()
            .trim()
            .is_empty()
    );
    assert!(
        String::from_utf8(
            repo.command(&["show", &format!("{}:new.txt", snapshot.commit)])
                .unwrap()
        )
        .unwrap()
        .contains("new")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn out_of_scope_changes_block_snapshot() {
    let (root, repo) = repo();
    let base = repo.head().unwrap();
    std::fs::write(root.join("base.txt"), "changed\n").unwrap();
    let index = root.parent().unwrap().join("ac-invalid-index");
    assert!(repo.snapshot(&base, &["allowed".into()], &index).is_err());
    assert_eq!(repo.head().unwrap(), base);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn partial_patch_conflict_preserves_destination_files_and_index() {
    let (root, repo) = repo();
    let base = repo.head().unwrap();
    std::fs::write(root.join("base.txt"), "agent partial\n").unwrap();
    let snapshot = repo
        .snapshot(
            &base,
            &["base.txt".into()],
            &root.with_extension("partial.index"),
        )
        .unwrap();
    let patch = repo.diff(&base, &snapshot.commit).unwrap();
    std::fs::write(root.join("base.txt"), "different dependency\n").unwrap();
    let before = repo.command(&["ls-files", "--stage", "-z"]).unwrap();
    assert!(repo.apply_working_patch(&patch).is_err());
    assert_eq!(
        std::fs::read_to_string(root.join("base.txt")).unwrap(),
        "different dependency\n"
    );
    assert_eq!(
        repo.command(&["ls-files", "--stage", "-z"]).unwrap(),
        before
    );
    assert_eq!(repo.head().unwrap(), base);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn worktree_is_optional_and_created_at_exact_base() {
    let (root, repo) = repo();
    let base = repo.head().unwrap();
    let path = root.with_extension("worktree");
    repo.worktree(&path, &base).unwrap();
    assert_eq!(Repository::open(&path).unwrap().head().unwrap(), base);
    repo.command(&["worktree", "remove", path.to_str().unwrap()])
        .unwrap();
    std::fs::remove_dir_all(root).unwrap();
}
