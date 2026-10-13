use agentconductor::slugs;
use std::path::Path;

#[test]
fn goal_folders_sit_directly_in_the_project_and_move_from_former_places() {
    let root = std::env::temp_dir().join(format!("ac-slug-folder-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (repo, workspace) = (root.join("project"), root.join("data"));
    std::fs::create_dir_all(&repo).unwrap();
    // User decision 2026-10-13: <project>/<slug>, e.g. C:\Temp\ladder\ladder_and_priority_games.
    assert_eq!(
        slugs::root(&repo, &workspace, "Ladder game", "ladder_game"),
        repo.join("ladder_game")
    );
    // Without a repository the slug stays in the workspace data folder.
    assert_eq!(
        slugs::root(Path::new(""), &workspace, "Ladder game", "x"),
        workspace.join("x")
    );
    for (name, folder) in [
        ("a/b:c*?", "a_b_c__"),
        ("  .hidden.  ", "hidden"),
        ("CON", "CON_"),
        (".git", "git"),
        ("node_modules", "workspace"),
        ("", "data"),
        ("사다리 게임", "사다리 게임"),
    ] {
        assert_eq!(slugs::folder_name(name, &workspace), folder, "{name:?}");
    }
    // The former <project>/.agentconductor/<slug> moves on first use.
    std::fs::create_dir_all(repo.join(".agentconductor/old_goal/plan")).unwrap();
    let moved = slugs::root(&repo, &workspace, "Ladder game", "old_goal");
    assert_eq!(moved, repo.join("old_goal"));
    assert!(moved.join("plan").is_dir());
    assert!(!repo.join(".agentconductor").exists());
    // The interim <project>/<workspace name>/<slug> moves too, and its emptied folder goes.
    std::fs::create_dir_all(repo.join("사다리 게임/ladder_and_priority_games/task")).unwrap();
    std::fs::write(repo.join("사다리 게임").join(slugs::MARKER), "x").unwrap();
    let moved = slugs::root(
        &repo,
        &workspace,
        "사다리 게임",
        "ladder_and_priority_games",
    );
    assert!(moved.join("task").is_dir());
    assert!(!repo.join("사다리 게임").exists());
    // A new tree is marked so folder copies skip it as project source.
    slugs::create_tree(&slugs::root(&repo, &workspace, "Ladder game", "new_goal")).unwrap();
    assert!(repo.join("new_goal").join(slugs::MARKER).is_file());
    assert!(repo.join("new_goal/verification").is_dir());
    std::fs::remove_dir_all(root).unwrap();
}
