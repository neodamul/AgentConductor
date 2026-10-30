use super::*;

#[test]
fn file_watch_detects_changes_and_backoff_is_bounded() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-watch-probe-{}",
        crate::native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("report.json"), b"before").unwrap();
    let mut watch = FileWatch::new(&root, UiSurface::AgentRoom);
    assert!(!watch.changed());
    fs::write(root.join("report.json"), b"different report length").unwrap();
    assert!(watch.changed());
    assert!(!watch.changed());
    assert_eq!(
        next_scan_delay(MIN_SCAN_DELAY, false),
        Duration::from_secs(2)
    );
    assert_eq!(next_scan_delay(MAX_SCAN_DELAY, false), MAX_SCAN_DELAY);
    assert_eq!(next_scan_delay(MAX_SCAN_DELAY, true), MIN_SCAN_DELAY);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn fallback_scan_includes_declared_worktree_files() {
    let root = std::env::temp_dir().join(format!(
        "agentconductor-scope-probe-{}",
        crate::native_guard::session_id().unwrap()
    ));
    let worktree = root.join("worktree");
    fs::create_dir_all(worktree.join("src")).unwrap();
    let target = worktree.join("src/external.rs");
    fs::write(&target, b"agent file").unwrap();
    fs::write(
        root.join("report.json"),
        serde_json::to_vec(&serde_json::json!({
            "plan": {"tasks": [{"id": "agent-1", "paths": ["src/**/*.rs"]}]},
            "tasks": [{"id": "agent-1", "cwd": worktree}]
        }))
        .unwrap(),
    )
    .unwrap();

    assert!(file_stamps(&root).iter().any(|(path, _)| path == &target));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn broad_glob_scopes_are_clamped_to_the_bounded_worktree_root() {
    assert_eq!(safe_scope_path("**/*"), Some(PathBuf::from(".")));
    assert_eq!(safe_scope_path("../outside/**"), None);
}
