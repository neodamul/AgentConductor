#[test]
fn oversized_settings_leave_no_pending_file_or_partial_update() {
    let root = std::env::temp_dir().join(format!(
        "ac-startup-write-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let path = root.join("startup.json");
    super::write(&path, &"original").unwrap();
    let original = std::fs::read(&path).unwrap();
    assert!(
        super::write(&path, &"x".repeat(65536))
            .unwrap_err()
            .contains("exceeds 64 KiB")
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
