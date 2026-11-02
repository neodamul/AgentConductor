use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn git(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit(repo: &Path, message: &str) {
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", message]);
}

fn check(repo: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_check_screenshots"))
        .arg(repo)
        .output()
        .unwrap()
}

fn repo() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ac-screenshots-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    git(&path, &["init", "-b", "main"]);
    git(&path, &["config", "user.email", "test@example.invalid"]);
    git(&path, &["config", "user.name", "Test"]);
    path
}

#[test]
fn a_capture_after_the_last_native_ui_change_passes() {
    let root = repo();
    fs::create_dir_all(root.join("src/native_ui_probe")).unwrap();
    fs::write(root.join("src/native_ui_probe/view.rs"), "first screen").unwrap();
    commit(&root, "native view");
    fs::create_dir_all(root.join("docs/assets")).unwrap();
    fs::write(root.join("docs/assets/ui-test-current.png"), "capture").unwrap();
    fs::write(
        root.join("docs/assets/screenshots-current.txt"),
        "Current screenshots\n- ui-test-current.png\n",
    )
    .unwrap();
    commit(&root, "capture screen");
    assert!(check(&root).status.success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_recaptured_image_after_a_native_ui_change_passes_without_manifest_edit() {
    let root = repo();
    fs::create_dir_all(root.join("src/native_ui_probe")).unwrap();
    fs::write(root.join("src/native_ui_probe/view.rs"), "first screen").unwrap();
    commit(&root, "native view");
    fs::create_dir_all(root.join("docs/assets")).unwrap();
    fs::write(root.join("docs/assets/ui-test-current.png"), "capture").unwrap();
    fs::write(
        root.join("docs/assets/screenshots-current.txt"),
        "Current screenshots\n- ui-test-current.png\n",
    )
    .unwrap();
    commit(&root, "capture screen");
    fs::write(root.join("src/native_ui_probe/view.rs"), "changed screen").unwrap();
    commit(&root, "change native view");
    fs::write(root.join("docs/assets/ui-test-current.png"), "recaptured").unwrap();
    commit(&root, "recapture screen without manifest edit");
    assert!(check(&root).status.success());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_later_native_ui_change_fails_until_screens_are_confirmed() {
    let root = repo();
    fs::create_dir_all(root.join("src/native_ui_probe")).unwrap();
    fs::write(root.join("src/native_ui_probe/view.rs"), "first screen").unwrap();
    commit(&root, "native view");
    fs::create_dir_all(root.join("docs/assets")).unwrap();
    fs::write(root.join("docs/assets/ui-test-current.png"), "capture").unwrap();
    fs::write(
        root.join("docs/assets/screenshots-current.txt"),
        "Current screenshots\n- ui-test-current.png\n",
    )
    .unwrap();
    commit(&root, "capture screen");
    fs::write(root.join("src/native_ui_probe/view.rs"), "changed screen").unwrap();
    commit(&root, "change native view");
    let stale = check(&root);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("native UI changed"));
    fs::write(
        root.join("docs/assets/screenshots-current.txt"),
        "Reviewed\n- ui-test-current.png\n",
    )
    .unwrap();
    commit(&root, "confirm current screenshots");
    assert!(check(&root).status.success());
    fs::remove_dir_all(root).unwrap();
}
