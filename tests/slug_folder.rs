use agentconductor::slugs;
use std::path::Path;

#[test]
fn goal_folders_sit_under_the_workspace_name_and_follow_moves() {
    let root = std::env::temp_dir().join(format!("ac-slug-folder-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (repo, workspace) = (root.join("project"), root.join("data"));
    std::fs::create_dir_all(&repo).unwrap();
    // User decision 2026-10-13: <project>/<workspace name>/<slug>.
    assert_eq!(
        slugs::root(&repo, &workspace, "Ladder game", "ladder_game"),
        repo.join("Ladder game").join("ladder_game")
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
    // A slug in the former <project>/.agentconductor/ is moved on first use.
    std::fs::create_dir_all(repo.join(".agentconductor/old_goal/plan")).unwrap();
    let moved = slugs::root(&repo, &workspace, "Ladder game", "old_goal");
    assert!(moved.join("plan").is_dir());
    assert!(!repo.join(".agentconductor").exists());
    // The marked folder follows a workspace rename and is skipped as project source.
    slugs::create_tree(&slugs::root(&repo, &workspace, "Ladder game", "new_goal")).unwrap();
    assert!(repo.join("Ladder game").join(slugs::MARKER).is_file());
    slugs::rename_folder(&repo, &workspace, "Ladder game", "Ladder");
    assert!(repo.join("Ladder/new_goal/task").is_dir());
    assert!(!repo.join("Ladder game").exists());
    std::fs::remove_dir_all(root).unwrap();
}
