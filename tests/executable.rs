//! Filesystem-only lookup preserves executable symlinks and refuses directories.
#[test]
fn lookup_accepts_files_without_spawning_them() {
    let dir = std::env::temp_dir().join(format!("ac-exe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let name = if cfg!(windows) { "tool.exe" } else { "tool" };
    let file = dir.join(name);
    std::fs::write(&file, b"never execute this fixture").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let dirs = vec![dir.clone()];
    assert_eq!(
        agentconductor::executable::resolve_in("tool", &dirs),
        Some(file.clone())
    );
    assert!(agentconductor::executable::resolve_in("missing", &dirs).is_none());
    assert!(agentconductor::executable::resolve_in(dir.to_str().unwrap(), &dirs).is_none());
    #[cfg(unix)]
    {
        let link = dir.join("claude");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        assert_eq!(
            agentconductor::executable::resolve_in("claude", &dirs),
            Some(link)
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}
