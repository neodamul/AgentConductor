//! #143: worktree-mode runs leave no linked worktrees behind once they are merged or
//! rejected, while a run that still needs its checkouts keeps them.
#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{git::Repository, workflow};

fn linked(repo: &std::path::Path) -> Vec<String> {
    let git = Repository::open(repo).unwrap();
    git.text(&["worktree", "list", "--porcelain"])
        .unwrap()
        .lines()
        .filter_map(|l| l.strip_prefix("worktree ").map(str::to_owned))
        .skip(1)
        .collect()
}

#[tokio::test]
async fn merging_removes_the_runs_worktrees_but_keeps_evidence() {
    let (root, plan) = support::setup("worktree");
    let (repo, run) = (root.join("repo"), root.join("run"));
    let report = workflow::run(plan, &repo, &run, 10).await.unwrap();
    // Waiting for approval still needs the task checkouts and the verification checkout.
    assert!(linked(&repo).len() >= 4, "{:?}", linked(&repo));
    workflow::approve(&run, &report.candidate.unwrap().commit).unwrap();
    assert!(linked(&repo).len() >= 4, "approved, not yet merged");

    workflow::merge(&run).unwrap();
    assert_eq!(linked(&repo), Vec::<String>::new());
    assert!(!run.join("workspace-0").exists());
    assert!(run.join("diff.patch").is_file() && run.join("report.json").is_file());
    assert_eq!(workflow::load(&run).unwrap().state, "completed");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn rejecting_removes_the_runs_worktrees() {
    let (root, plan) = support::setup("worktree");
    let (repo, run) = (root.join("repo"), root.join("run"));
    workflow::run(plan, &repo, &run, 10).await.unwrap();
    workflow::reject(&run, "not wanted").unwrap();
    assert_eq!(linked(&repo), Vec::<String>::new());
    assert!(run.join("diff.patch").is_file());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn dangling_entries_are_pruned_and_other_worktrees_are_left_alone() {
    let (root, plan) = support::setup("worktree");
    let (repo, run) = (root.join("repo"), root.join("run"));
    workflow::run(plan, &repo, &run, 10).await.unwrap();
    let git = Repository::open(&repo).unwrap();
    let other = root.join("someone-elses-worktree");
    git.worktree(&other, &git.head().unwrap()).unwrap();
    // The run folder checkouts vanish behind git's back.
    std::fs::remove_dir_all(run.join("workspace-0")).unwrap();

    assert!(git.remove_worktrees_under(&run).unwrap() >= 2);
    let left = linked(&repo);
    assert_eq!(left.len(), 1, "{left:?}");
    assert!(left[0].ends_with("someone-elses-worktree"));
    std::fs::remove_dir_all(root).unwrap();
}

/// Deleting a workspace or clearing its cache releases a run's worktrees first.
#[tokio::test]
async fn releasing_a_run_before_it_is_deleted_leaves_the_repository_clean() {
    let (root, plan) = support::setup("worktree");
    let (repo, run) = (root.join("repo"), root.join("run"));
    workflow::run(plan, &repo, &run, 10).await.unwrap();
    assert!(!linked(&repo).is_empty());
    workflow::release_run_worktrees(&run);
    assert_eq!(linked(&repo), Vec::<String>::new());
    workflow::release_run_worktrees(&root.join("no-such-run"));
    std::fs::remove_dir_all(root).unwrap();
}
