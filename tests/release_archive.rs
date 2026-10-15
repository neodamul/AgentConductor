use agentconductor::release_archive::archive_package;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
use zip::ZipArchive;

fn fixture() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ac-release-archive-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn archive_preserves_contents_executable_mode_and_clamps_old_timestamps() {
    let root = fixture();
    let package = root.join("package");
    fs::create_dir(&package).unwrap();
    fs::create_dir(package.join("nested")).unwrap();
    let binary = package.join("agentconductor");
    fs::write(&binary, b"exact binary").unwrap();
    fs::write(package.join("nested/info.txt"), b"release notes").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        fs::File::options()
            .write(true)
            .open(&binary)
            .unwrap()
            .set_times(
                fs::FileTimes::new()
                    .set_accessed(std::time::UNIX_EPOCH)
                    .set_modified(std::time::UNIX_EPOCH),
            )
            .unwrap();
    }

    let archive_path = archive_package(&package).unwrap();
    let mut archive = ZipArchive::new(fs::File::open(archive_path).unwrap()).unwrap();
    let mut binary_entry = archive.by_name("package/agentconductor").unwrap();
    use std::io::Read;
    let mut contents = Vec::new();
    binary_entry.read_to_end(&mut contents).unwrap();
    assert_eq!(contents, b"exact binary");
    assert_eq!(binary_entry.last_modified().unwrap().year(), 1980);
    #[cfg(unix)]
    {
        let mode = binary_entry.unix_mode().unwrap();
        assert_ne!(mode & 0o111, 0);
        assert_eq!(mode & 0o170000, 0o100000);
    }
    drop(binary_entry);
    assert!(archive.by_name("package/nested/info.txt").is_ok());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_and_existing_outputs_without_leaving_partial_archives() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    let package = root.join("package");
    fs::create_dir(&package).unwrap();
    fs::write(package.join("file"), b"data").unwrap();
    symlink(package.join("file"), package.join("link")).unwrap();
    let archive = package.with_extension("zip");
    let pending = package.with_extension("zip.pending");

    assert!(archive_package(&package).is_err());
    assert!(!archive.exists());
    assert!(!pending.exists());
    fs::remove_file(package.join("link")).unwrap();
    fs::write(&pending, b"reserved").unwrap();
    assert!(archive_package(&package).is_err());
    assert_eq!(fs::read(&pending).unwrap(), b"reserved");

    fs::remove_file(pending).unwrap();
    archive_package(&package).unwrap();
    assert!(archive_package(&package).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hidden_cli_archives_package_and_prints_archive_path() {
    let root = fixture();
    let package = root.join("package");
    fs::create_dir(&package).unwrap();
    fs::write(package.join("binary"), b"payload").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("archive-package")
        .arg(&package)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = package.canonicalize().unwrap().with_extension("zip");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        expected.display().to_string()
    );
    fs::remove_dir_all(root).unwrap();
}
