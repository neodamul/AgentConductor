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

#[test]
fn failed_smoke_removes_its_temporary_workspace() {
    let root = fixture();
    let binary_name = if cfg!(windows) {
        "agentconductor.exe"
    } else {
        "agentconductor"
    };
    let directory = package(&root, binary_name, b"placeholder");
    let source = root.join("fail.rs");
    fs::write(&source, "fn main() { std::process::exit(23); }\n").unwrap();
    let binary = directory.join(binary_name);
    fs::remove_file(&binary).unwrap();
    let compile = Command::new("rustc")
        .args([source.to_str().unwrap(), "-o", binary.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(compile.success());
    let bytes = fs::read(&binary).unwrap();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec(&json!({
            "binary": binary_name,
            "binary_sha256": digest,
            "source_commit": "a".repeat(40)
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("SHA256SUMS"),
        format!("{digest}  {binary_name}\n"),
    )
    .unwrap();

    let prefix = format!("ac-release-smoke-{}-", std::process::id());
    let before = fs::read_dir(std::env::temp_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .collect::<Vec<_>>();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let result = runtime.block_on(agentconductor::release_smoke::run_smoke(
        &directory,
        &root.join("evidence"),
    ));
    assert!(result.is_err());
    let leaked = fs::read_dir(std::env::temp_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .filter(|path| !before.contains(path))
        .collect::<Vec<_>>();
    for path in &leaked {
        fs::remove_dir_all(path).unwrap();
    }
    fs::remove_dir_all(root).unwrap();
    assert!(leaked.is_empty(), "failed smoke left {leaked:?}");
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

#[test]
fn hidden_cli_runs_packaged_release_acceptance() {
    let root = fixture();
    let binary_name = if cfg!(windows) {
        "agentconductor.exe"
    } else {
        "agentconductor"
    };
    let directory = package(&root, binary_name, b"placeholder");
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_agentconductor"));
    let destination = directory.join(binary_name);
    fs::copy(&binary, &destination).unwrap();
    let bytes = fs::read(&destination).unwrap();
    let digest = format!("{:x}", Sha256::digest(&bytes));
    fs::write(
        directory.join("manifest.json"),
        serde_json::to_vec(&json!({
            "binary": binary_name,
            "binary_sha256": digest,
            "source_commit": "a".repeat(40)
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        directory.join("SHA256SUMS"),
        format!("{digest}  {binary_name}\n"),
    )
    .unwrap();

    let evidence = root.join("evidence");
    for _ in 0..2 {
        let output = Command::new(&binary)
            .args([
                "smoke-package",
                directory.to_str().unwrap(),
                evidence.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["cases"].as_array().unwrap().len(), 4);
        assert!(result["cases"].as_array().unwrap().iter().all(|case| {
            case["tasks"] == 3
                && case["unapproved_merge_refused"] == true
                && case["approved_merge_verified"] == true
        }));
    }
    let runs = fs::read_dir(&evidence)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    assert_eq!(runs.len(), 2);
    assert!(runs.iter().all(|run| run.join("smoke.json").is_file()));
    fs::remove_dir_all(root).unwrap();
}
