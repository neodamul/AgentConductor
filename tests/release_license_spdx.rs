#[path = "support/release_package.rs"]
mod release_package_support;

use agentconductor::release_package::assemble;
use release_package_support::{fixture, temp};
use serde_json::json;
use std::{fs, path::Path};

#[test]
fn packages_mit_license_choices_with_reviewed_mit_text() {
    let root = temp();
    let target = "aarch64-apple-darwin";
    let mut metadata = fixture(&root, target);
    let dependencies = [
        (
            "block2",
            "0.5.1",
            "MIT",
            "4fc083f1c6d6784577e38b0ee8dbd344481e2fd2",
        ),
        (
            "block2",
            "0.6.2",
            "MIT",
            "b4167b582b2f75f9a1be75495c41b765344fd03c",
        ),
        (
            "dispatch",
            "0.2.0",
            "MIT",
            "82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748",
        ),
        (
            "dispatch2",
            "0.3.1",
            "Zlib OR Apache-2.0 OR MIT",
            "8852b424193ca41602281b3d7540d7c8ed51e49a",
        ),
        (
            "ecolor",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
        ),
        (
            "eframe",
            "0.37.0",
            "MIT OR Apache-2.0",
            "3c6eb8261c3293ad9c7ea1ed58487e038860ebef",
        ),
        (
            "egui",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
        ),
        (
            "egui-winit",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
        ),
        (
            "egui_glow",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
        ),
        (
            "egui_system_font_provider",
            "0.37.0",
            "MIT OR Apache-2.0",
            "3c6eb8261c3293ad9c7ea1ed58487e038860ebef",
        ),
        (
            "emath",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
        ),
        (
            "epaint",
            "0.37.0",
            "MIT OR Apache-2.0",
            "7c0b40402271e8d40cda69f87cf3c2617c2e4382",
        ),
        (
            "objc2",
            "0.5.3",
            "MIT",
            "5237cbf081d04ef75a1f6a207b4018e7c1ae7438",
        ),
        (
            "objc2",
            "0.6.5",
            "MIT",
            "d7d2fa23ceaa5e6096c923b081040e5d81b3b9df",
        ),
        (
            "objc2-app-kit",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
        ),
        (
            "objc2-app-kit",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
        ),
        (
            "objc2-core-data",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
        ),
        (
            "objc2-core-foundation",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
        ),
        (
            "objc2-core-graphics",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
        ),
        (
            "objc2-core-image",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
        ),
        (
            "objc2-core-text",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
        ),
        (
            "objc2-encode",
            "4.1.0",
            "MIT",
            "8d214f5477365ffcbcbb7de058c86ed9a518efb7",
        ),
        (
            "objc2-foundation",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
        ),
        (
            "objc2-foundation",
            "0.3.2",
            "MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
        ),
        (
            "objc2-io-surface",
            "0.3.2",
            "Zlib OR Apache-2.0 OR MIT",
            "7b1abfd750a2cacaea71d6a56ecfb83cb7de560b",
        ),
        (
            "objc2-metal",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
        ),
        (
            "objc2-quartz-core",
            "0.2.2",
            "MIT",
            "e282618be4c3a3b9542957e0c8540e9588472ce8",
        ),
        (
            "objc-sys",
            "0.3.5",
            "MIT",
            "4fc083f1c6d6784577e38b0ee8dbd344481e2fd2",
        ),
        (
            "profiling",
            "1.0.18",
            "MIT OR Apache-2.0",
            "8271551172eb6fa4cba47369aedd93790c623df9",
        ),
    ];
    for (name, version, license, revision) in dependencies {
        let source = root.join(format!("{name}-{version}"));
        fs::create_dir_all(&source).unwrap();
        let manifest = source.join("Cargo.toml");
        fs::write(&manifest, format!("[package]\nname = \"{name}\"\n")).unwrap();
        fs::write(
            source.join(".cargo_vcs_info.json"),
            json!({"git":{"sha1":revision}}).to_string(),
        )
        .unwrap();
        let id = format!("registry+fixture#{name}@{version}");
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":id}));
        metadata["packages"].as_array_mut().unwrap().push(json!({
            "id":id,
            "source":"registry+fixture",
            "name":name,
            "version":version,
            "license":license,
            "manifest_path":manifest
        }));
    }
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("licenses/upstream");
    fs::create_dir_all(upstream.join("spdx")).unwrap();
    let provenance_path = project.join("licenses/upstream/sources.json");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(&provenance_path).unwrap()).unwrap();
    fs::copy(&provenance_path, upstream.join("sources.json")).unwrap();
    for (name, version, license, _) in &dependencies {
        let Some(record) = provenance.as_array().unwrap().iter().find(|record| {
            record["name"] == *name
                && record["version"] == *version
                && record["license"] == *license
        }) else {
            continue;
        };
        let relative = record["file"].as_str().unwrap();
        let destination = upstream.join(relative);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(
            project.join("licenses/upstream").join(relative),
            destination,
        )
        .unwrap();
    }
    fs::copy(
        project.join("licenses/upstream/spdx/LICENSE-MIT"),
        upstream.join("spdx/LICENSE-MIT"),
    )
    .unwrap();

    let destination = root.join("dist/package");
    let result = assemble(&root, &destination, &metadata, target, &"f".repeat(40));
    assert!(result.is_ok(), "{}", result.unwrap_err());
    for (name, version, license, _) in dependencies {
        let package_dir = destination.join(format!("licenses/{name}-{version}"));
        let actual = fs::read_dir(package_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("LICENSE"))
            })
            .expect("packaged license text missing");
        let expected = provenance
            .as_array()
            .unwrap()
            .iter()
            .find(|record| {
                record["name"] == name
                    && record["version"] == version
                    && record["license"] == license
            })
            .map(|record| {
                project
                    .join("licenses/upstream")
                    .join(record["file"].as_str().unwrap())
            })
            .unwrap_or_else(|| project.join("licenses/upstream/spdx/LICENSE-MIT"));
        let bytes = fs::read(actual).unwrap();
        assert_eq!(bytes, fs::read(expected).unwrap());
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains("Copyright (c) <year> <copyright holders>"),
            "placeholder license packaged for {name} {version}"
        );
    }
    fs::remove_dir_all(root).unwrap();
}
