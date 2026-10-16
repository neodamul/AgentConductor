#[path = "support/release_package.rs"]
mod release_package_support;

use agentconductor::release_package::assemble;
use release_package_support::{fixture, temp};
use serde_json::json;
use std::{fs, path::Path};

#[test]
fn packages_both_block2_versions_with_reviewed_mit_text() {
    let root = temp();
    let target = "aarch64-apple-darwin";
    let mut metadata = fixture(&root, target);
    let dependencies = [
        ("0.5.1", "4fc083f1c6d6784577e38b0ee8dbd344481e2fd2"),
        ("0.6.2", "b4167b582b2f75f9a1be75495c41b765344fd03c"),
    ];
    for (version, revision) in dependencies {
        let source = root.join(format!("block2-{version}"));
        fs::create_dir_all(&source).unwrap();
        let manifest = source.join("Cargo.toml");
        fs::write(&manifest, "[package]\nname = \"block2\"\n").unwrap();
        fs::write(
            source.join(".cargo_vcs_info.json"),
            json!({"git":{"sha1":revision}}).to_string(),
        )
        .unwrap();
        let id = format!("registry+fixture#block2@{version}");
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":id}));
        metadata["packages"].as_array_mut().unwrap().push(json!({
            "id":id,
            "source":"registry+fixture",
            "name":"block2",
            "version":version,
            "license":"MIT",
            "manifest_path":manifest
        }));
    }
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("licenses/upstream");
    fs::create_dir_all(upstream.join("spdx")).unwrap();
    fs::copy(
        project.join("licenses/upstream/sources.json"),
        upstream.join("sources.json"),
    )
    .unwrap();
    fs::copy(
        project.join("licenses/upstream/spdx/LICENSE-MIT"),
        upstream.join("spdx/LICENSE-MIT"),
    )
    .unwrap();

    let destination = root.join("dist/package");
    let result = assemble(&root, &destination, &metadata, target, &"f".repeat(40));
    assert!(result.is_ok(), "{}", result.unwrap_err());
    for (version, _) in dependencies {
        let license =
            fs::read_to_string(destination.join(format!("licenses/block2-{version}/LICENSE-MIT")))
                .unwrap();
        assert_eq!(
            license,
            fs::read_to_string(project.join("licenses/upstream/spdx/LICENSE-MIT")).unwrap()
        );
    }
    fs::remove_dir_all(root).unwrap();
}
