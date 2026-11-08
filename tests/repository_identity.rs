//! GitHub #562: the footer names the Workspace folder and its branch from `.git/HEAD`.
use agentconductor::ui::setup::repository_identity;
use std::path::Path;

fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ac-identity-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_footer_reads_folder_and_branch_without_running_git() {
    let repo = folder("repo").join("auth-service");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/feature/rotation\n").unwrap();
    let id = repository_identity(&repo);
    assert_eq!(
        (id["name"].as_str(), id["branch"].as_str()),
        (Some("auth-service"), Some("feature/rotation"))
    );
    std::fs::write(
        repo.join(".git/HEAD"),
        "0123456789abcdef0123456789abcdef01234567\n",
    )
    .unwrap();
    assert_eq!(repository_identity(&repo)["branch"], "0123456");
    // A linked worktree points at its own git directory.
    let linked = repo.parent().unwrap().join("linked");
    std::fs::create_dir_all(linked.join("meta")).unwrap();
    std::fs::write(linked.join(".git"), "gitdir: meta\n").unwrap();
    std::fs::write(linked.join("meta/HEAD"), "ref: refs/heads/topic\n").unwrap();
    assert_eq!(repository_identity(&linked)["branch"], "topic");
    // Outside Git there is no branch; with no folder chosen there is nothing to show.
    let plain = folder("plain");
    assert!(repository_identity(&plain)["branch"].is_null());
    assert!(repository_identity(Path::new("")).is_null());
}
