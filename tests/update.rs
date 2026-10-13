//! DD-135: release comparison, checksum verification and replacing the running executable.
use agentconductor::update;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ac-update-{name}-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn only_a_strictly_newer_release_counts() {
    let current = update::current();
    let mut parts: Vec<u64> = current.split('.').map(|p| p.parse().unwrap()).collect();
    assert!(!update::is_newer(&format!("v{current}")));
    parts[2] += 1;
    assert!(update::is_newer(&format!(
        "v{}.{}.{}",
        parts[0], parts[1], parts[2]
    )));
    assert!(!update::is_newer("v0.0.1"));
    assert!(!update::is_newer("nightly"));
}

#[test]
fn the_archive_must_match_its_published_checksum() {
    use sha2::{Digest, Sha256};
    let dir = scratch("verify");
    let archive = dir.join("agentconductor-x86_64-pc-windows-msvc-abc.zip");
    std::fs::write(&archive, b"package bytes").unwrap();
    let hash = format!("{:x}", Sha256::digest(b"package bytes"));
    let sums = format!(
        "{}  agentconductor-aarch64-apple-darwin-abc.zip\n{hash}  agentconductor-x86_64-pc-windows-msvc-abc.zip\n",
        "0".repeat(64)
    );
    let name = "agentconductor-x86_64-pc-windows-msvc-abc.zip";
    update::verify(&archive, name, &sums).unwrap();
    std::fs::write(&archive, b"tampered").unwrap();
    assert!(update::verify(&archive, name, &sums).is_err());
    assert!(update::verify(&archive, "missing.zip", &sums).is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_running_executable_is_moved_aside_and_replaced() {
    let dir = scratch("replace");
    let binary = if cfg!(windows) {
        "agentconductor.exe"
    } else {
        "agentconductor"
    };
    let installed = dir.join("app").join(binary);
    std::fs::create_dir_all(installed.parent().unwrap()).unwrap();
    std::fs::write(&installed, b"old build").unwrap();
    let package = dir.join("unpacked/agentconductor-target-abc");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join(binary), b"new build").unwrap();

    update::replace(&installed, &package).unwrap();
    assert_eq!(std::fs::read(&installed).unwrap(), b"new build");
    assert_eq!(
        std::fs::read(update::leftover(&installed)).unwrap(),
        b"old build"
    );

    // A package without the executable leaves the installed one untouched.
    let empty = dir.join("unpacked/empty");
    std::fs::create_dir_all(&empty).unwrap();
    assert!(update::replace(&installed, &empty).is_err());
    assert_eq!(std::fs::read(&installed).unwrap(), b"new build");
    std::fs::remove_dir_all(dir).unwrap();
}
