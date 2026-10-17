use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[test]
fn windows_clipboard_boost_license_is_pinned_to_the_crate_source_revision() {
    let sources: serde_json::Value =
        serde_json::from_str(include_str!("../licenses/upstream/sources.json")).unwrap();
    let record = sources
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["name"] == "clipboard-win" && record["version"] == "5.4.1")
        .unwrap();
    assert_eq!(record["license"], "BSL-1.0");
    let revision = record["vcs_commit"].as_str().unwrap();
    assert_eq!(revision, "3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342");
    let relative = record["file"].as_str().unwrap();
    assert!(relative.starts_with(&format!("{revision}/")));
    let bytes = std::fs::read(format!("licenses/upstream/{relative}")).unwrap();
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), record["sha256"]);
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .starts_with("Boost Software License - Version 1.0")
    );
}

#[test]
fn egui_wgpu_mit_license_is_pinned_to_the_crate_source_revision() {
    let sources: serde_json::Value =
        serde_json::from_str(include_str!("../licenses/upstream/sources.json")).unwrap();
    let record = sources
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["name"] == "egui-wgpu" && record["version"] == "0.37.0")
        .unwrap();
    assert_eq!(record["license"], "MIT OR Apache-2.0");
    let revision = "7c0b40402271e8d40cda69f87cf3c2617c2e4382";
    assert_eq!(record["vcs_commit"], revision);
    assert_eq!(
        record["license_source_commit"],
        "841a328ba2dd8f6996beb7189ebb7fc00e668a2a"
    );
    let relative = record["file"].as_str().unwrap();
    assert_eq!(relative, format!("{revision}/LICENSE-MIT"));
    let bytes = std::fs::read(format!("licenses/upstream/{relative}")).unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "95ca92f5f8ea5231f1580b3a2a799e8260af3114b900e1def5355a7f44bcf60c"
    );
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .contains("Copyright (c) 2018-2021 Emil Ernerfeldt")
    );
}

#[test]
fn native_package_uses_the_reviewed_egui_wgpu_license() {
    use agentconductor::release_package::assemble;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ac-egui-license-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("README.md"), "# Fixture\n").unwrap();
    fs::write(root.join("Cargo.lock"), "locked\n").unwrap();
    fs::write(root.join("LICENSE"), "MIT fixture\n").unwrap();
    let binary = root.join("target/release/agentconductor.exe");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, b"fixture").unwrap();

    let source = root.join("dependency");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[package]\nname = \"egui-wgpu\"\n",
    )
    .unwrap();
    fs::write(
        source.join(".cargo_vcs_info.json"),
        json!({"git":{"sha1":"7c0b40402271e8d40cda69f87cf3c2617c2e4382"}}).to_string(),
    )
    .unwrap();

    let licenses = root.join("licenses/upstream");
    fs::create_dir_all(licenses.join("7c0b40402271e8d40cda69f87cf3c2617c2e4382")).unwrap();
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::copy(
        project.join("licenses/upstream/sources.json"),
        licenses.join("sources.json"),
    )
    .unwrap();
    fs::copy(
        project.join("licenses/upstream/7c0b40402271e8d40cda69f87cf3c2617c2e4382/LICENSE-MIT"),
        licenses.join("7c0b40402271e8d40cda69f87cf3c2617c2e4382/LICENSE-MIT"),
    )
    .unwrap();

    let metadata = json!({
        "resolve":{"nodes":[{"id":"egui-wgpu-id"}]},
        "packages":[{"id":"egui-wgpu-id","source":"registry+fixture","name":"egui-wgpu","version":"0.37.0","license":"MIT OR Apache-2.0","manifest_path":source.join("Cargo.toml")}]
    });
    let destination = root.join("dist/package");
    assemble(
        &root,
        &destination,
        &metadata,
        "x86_64-pc-windows-msvc",
        &"a".repeat(40),
    )
    .unwrap();
    assert_eq!(
        fs::read(destination.join("licenses/egui-wgpu-0.37.0/LICENSE-MIT")).unwrap(),
        fs::read(
            project.join("licenses/upstream/7c0b40402271e8d40cda69f87cf3c2617c2e4382/LICENSE-MIT")
        )
        .unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}
