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
fn native_package_includes_each_reviewed_macos_license() {
    use agentconductor::release_package::assemble;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!(
        "ac-native-license-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("README.md"), "# Fixture\n").unwrap();
    fs::write(root.join("docs/user-manual.md"), "# Fixture manual\n").unwrap();
    fs::write(root.join("docs/user-manual.ko.md"), "# Fixture manual\n").unwrap();
    fs::write(root.join("Cargo.lock"), "locked\n").unwrap();
    fs::write(root.join("company-suffix.txt"), "\n").unwrap();
    fs::write(root.join("LICENSE"), "MIT fixture\n").unwrap();
    let binary = root.join("target/release/agentconductor.exe");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, b"fixture").unwrap();

    let dependencies = [
        (
            "egui-wgpu",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
            "LICENSE-MIT",
        ),
        (
            "objc2-metal",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
            "LICENSE.txt",
        ),
        (
            "objc2-metal",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
            "LICENSE.md",
        ),
        (
            "objc2-quartz-core",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
            "LICENSE.md",
        ),
        (
            "spirv",
            "0.4.0+sdk-1.4.341.0",
            "Apache-2.0",
            "8afc3d0ac8e158128cd1410bb2e4b4c26ab11bb4",
            "LICENSE",
        ),
        (
            "pollster",
            "1.0.1",
            "Apache-2.0/MIT",
            "local",
            "LICENSE-MIT",
        ),
        (
            "rustc-hash",
            "1.1.0",
            "Apache-2.0/MIT",
            "local",
            "LICENSE-MIT",
        ),
    ];
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("licenses/upstream");
    fs::create_dir_all(&upstream).unwrap();
    fs::copy(
        project.join("licenses/upstream/sources.json"),
        upstream.join("sources.json"),
    )
    .unwrap();
    let mut nodes = Vec::new();
    let mut packages = Vec::new();
    for (index, (name, version, license, revision, file)) in dependencies.iter().enumerate() {
        let id = format!("dependency-{index}");
        let source = root.join(&id);
        fs::create_dir_all(&source).unwrap();
        fs::write(
            source.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\n"),
        )
        .unwrap();
        fs::write(
            source.join(".cargo_vcs_info.json"),
            json!({"git":{"sha1":revision}}).to_string(),
        )
        .unwrap();
        nodes.push(json!({"id":id.clone()}));
        packages.push(json!({
            "id":id,
            "source":"registry+fixture",
            "name":name,
            "version":version,
            "license":license,
            "manifest_path":source.join("Cargo.toml")
        }));
        if *revision == "local" {
            fs::write(source.join(file), "dual-license fixture\n").unwrap();
        } else {
            let directory = upstream.join(revision);
            fs::create_dir_all(&directory).unwrap();
            fs::copy(
                project.join(format!("licenses/upstream/{revision}/{file}")),
                directory.join(file),
            )
            .unwrap();
        }
    }
    let metadata = json!({"resolve":{"nodes":nodes},"packages":packages});
    let destination = root.join("dist/package");
    assemble(
        &root,
        &destination,
        &metadata,
        "x86_64-pc-windows-msvc",
        &"a".repeat(40),
    )
    .unwrap();
    for (name, version, _, revision, file) in dependencies {
        let relative = format!("licenses/{name}-{version}/{file}");
        let expected = if revision == "local" {
            b"dual-license fixture\n".to_vec()
        } else {
            fs::read(project.join(format!("licenses/upstream/{revision}/{file}"))).unwrap()
        };
        assert_eq!(fs::read(destination.join(&relative)).unwrap(), expected);
    }
    let sbom: serde_json::Value =
        serde_json::from_slice(&fs::read(destination.join("SBOM.cdx.json")).unwrap()).unwrap();
    for name in ["pollster", "rustc-hash"] {
        let component = sbom["components"]
            .as_array()
            .unwrap()
            .iter()
            .find(|component| component["name"] == name)
            .unwrap();
        assert_eq!(component["licenses"][0]["expression"], "Apache-2.0 OR MIT");
    }
    fs::remove_dir_all(root).unwrap();
}
