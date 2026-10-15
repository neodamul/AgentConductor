use agentconductor::release_smoke::validate_package;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

fn fixture() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ac-release-smoke-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

fn package(root: &Path, binary: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join("package");
    fs::create_dir(&path).unwrap();
    fs::write(path.join(binary), bytes).unwrap();
    let digest = format!("{:x}", Sha256::digest(bytes));
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec(&json!({
            "binary": binary,
            "binary_sha256": digest,
            "source_commit": "a".repeat(40)
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(path.join("SHA256SUMS"), format!("{digest}  {binary}\n")).unwrap();
    path
}

#[test]
fn validates_exact_binary_manifest_and_checksum_before_execution() {
    let root = fixture();
    let directory = package(&root, "agentconductor", b"fixture");
    let validated = validate_package(&directory).unwrap();
    assert_eq!(
        validated.binary,
        directory.canonicalize().unwrap().join("agentconductor")
    );
    assert_eq!(validated.manifest["source_commit"], "a".repeat(40));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_bad_hash_manifest_and_escaping_binary_names() {
    let root = fixture();
    let directory = package(&root, "agentconductor", b"fixture");
    fs::write(directory.join("agentconductor"), b"changed").unwrap();
    assert!(validate_package(&directory).is_err());
    fs::remove_dir_all(directory).unwrap();

    let directory = package(&root, "../agentconductor", b"fixture");
    assert!(validate_package(&directory).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_invalid_source_commit_and_mismatched_checksum_file() {
    let root = fixture();
    let directory = package(&root, "agentconductor", b"fixture");
    let manifest_path = directory.join("manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["source_commit"] = json!("not-a-commit");
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert!(validate_package(&directory).is_err());

    manifest["source_commit"] = json!("a".repeat(40));
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    fs::write(directory.join("SHA256SUMS"), "wrong checksum\n").unwrap();
    assert!(validate_package(&directory).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn rejects_symbolic_link_binaries() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    let directory = root.join("package");
    fs::create_dir(&directory).unwrap();
    fs::write(root.join("outside"), b"fixture").unwrap();
    symlink(root.join("outside"), directory.join("agentconductor")).unwrap();
    let digest = format!("{:x}", Sha256::digest(b"fixture"));
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec(&json!({
            "binary": "agentconductor",
            "binary_sha256": digest,
            "source_commit": "a".repeat(40)
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("SHA256SUMS"),
        format!("{digest}  agentconductor\n"),
    )
    .unwrap();
    assert!(validate_package(&directory).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn hidden_cli_returns_the_validated_manifest_and_binary_path() {
    let root = fixture();
    let directory = package(&root, "agentconductor", b"fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("validate-package")
        .arg(&directory)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(response["manifest"]["binary"], "agentconductor");
    let expected = directory.canonicalize().unwrap().join("agentconductor");
    assert_eq!(response["binary"], expected.to_string_lossy().as_ref());
    fs::remove_dir_all(root).unwrap();
}
