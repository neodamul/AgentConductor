use agentconductor::release_package::{assemble, assemble_with_binary};
#[path = "support/release_package.rs"]
mod release_package_support;
use release_package_support::{fixture, temp};
use sha2::{Digest, Sha256};
use std::fs;

#[test]
fn assembles_macos_and_windows_packages_with_reviewed_dependencies() {
    for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"] {
        let root = temp();
        let metadata = fixture(&root, target);
        let name = if target.contains("windows") {
            "agentconductor.exe"
        } else {
            "agentconductor"
        };
        let binary = root.join("target/release").join(name);
        let destination = root.join("dist/package");
        let manifest = assemble_with_binary(
            &root,
            &destination,
            &metadata,
            target,
            &"a".repeat(40),
            &binary,
        )
        .unwrap();
        assert_eq!(
            fs::read(destination.join(name)).unwrap(),
            b"exact native binary"
        );
        assert_eq!(manifest["source_commit"], "a".repeat(40));
        assert_eq!(
            manifest["binary_sha256"],
            format!("{:x}", Sha256::digest(b"exact native binary"))
        );
        assert_eq!(manifest["dependency_count"], 1);
        assert_eq!(manifest["binary_bytes"], b"exact native binary".len());
        assert!(destination.join("licenses/dep-1.0.0/LICENSE-MIT").is_file());
        assert_eq!(
            fs::read(destination.join("LICENSE")).unwrap(),
            b"MIT License\n\nCopyright (c) fixture\n"
        );
        assert_eq!(
            fs::read(destination.join("Cargo.lock")).unwrap(),
            b"locked fixture\n"
        );
        assert_eq!(
            manifest["external_requirements"],
            serde_json::json!(["Git", "selected authenticated agent CLIs"])
        );
        assert!(destination.join("docs/assets/live.jpg").is_file());
        assert!(!destination.join("docs/assets/mock.jpg").exists());
        assert!(destination.join("SBOM.cdx.json").is_file());
        assert!(destination.join("DEPENDENCY-LICENSES.md").is_file());
        assert!(destination.join("THIRD-PARTY-NOTICES.md").is_file());
        assert!(
            fs::read(destination.join("SHA256SUMS"))
                .unwrap()
                .ends_with(format!("  {name}\n").as_bytes())
        );
        if target.contains("apple") {
            assert_eq!(
                fs::read(destination.join("AgentConductor.app/Contents/MacOS/agentconductor"))
                    .unwrap(),
                b"exact native binary"
            );
        }
        for file in [
            "SHA256SUMS",
            "manifest.json",
            "SBOM.cdx.json",
            "DEPENDENCY-LICENSES.md",
            "THIRD-PARTY-NOTICES.md",
        ] {
            assert!(
                !fs::read(destination.join(file))
                    .unwrap()
                    .windows(2)
                    .any(|pair| pair == b"\r\n")
            );
        }
        assert!(assemble(&root, &destination, &metadata, target, &"a".repeat(40)).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn package_uses_the_compiled_binary_instead_of_a_stale_repository_target() {
    let root = temp();
    let target = "aarch64-apple-darwin";
    let metadata = fixture(&root, target);
    let cached = root.join("target/release/agentconductor");
    fs::write(&cached, b"stale cached binary").unwrap();
    let compiled = root.join("custom-target/release/agentconductor");
    fs::create_dir_all(compiled.parent().unwrap()).unwrap();
    fs::write(&compiled, b"current package-release binary").unwrap();
    let destination = root.join("dist/package");

    let manifest = assemble_with_binary(
        &root,
        &destination,
        &metadata,
        target,
        &"b".repeat(40),
        &compiled,
    )
    .unwrap();

    assert_eq!(
        fs::read(destination.join("agentconductor")).unwrap(),
        b"current package-release binary"
    );
    assert_eq!(
        fs::read(destination.join("AgentConductor.app/Contents/MacOS/agentconductor")).unwrap(),
        b"current package-release binary"
    );
    assert_eq!(
        manifest["binary_sha256"],
        format!("{:x}", Sha256::digest(b"current package-release binary"))
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn removes_partial_package_when_a_required_input_is_missing() {
    let root = temp();
    let target = "x86_64-pc-windows-msvc";
    let metadata = fixture(&root, target);
    fs::remove_file(root.join("Cargo.lock")).unwrap();
    let destination = root.join("dist/package");
    assert!(assemble(&root, &destination, &metadata, target, &"e".repeat(40)).is_err());
    assert!(!destination.exists());
    assert!(
        fs::read_dir(destination.parent().unwrap())
            .unwrap()
            .next()
            .is_none()
    );
    fs::remove_dir_all(root).unwrap();
}
