#[path = "support/release_package.rs"]
mod release_package_support;

use agentconductor::release_package::assemble;
use release_package_support::{fixture, temp};
use serde_json::json;
use std::{fs, path::Path};

#[test]
fn packages_gl_generator_crates_with_pinned_apache_text() {
    let root = temp();
    let target = "x86_64-pc-windows-msvc";
    let mut metadata = fixture(&root, target);
    let dependencies = [
        (
            "gl_generator",
            "0.14.0",
            "ea503e8d5fb6d73c6030e6191ce738cd3bf3433e",
        ),
        (
            "khronos_api",
            "3.1.0",
            "f150967b1c44ae888e6676f93f639ebc82771bdc",
        ),
    ];
    for (name, version, revision) in dependencies {
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
            "license":"Apache-2.0",
            "manifest_path":manifest
        }));
    }
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("licenses/upstream");
    fs::create_dir_all(&upstream).unwrap();
    fs::copy(
        project.join("licenses/upstream/sources.json"),
        upstream.join("sources.json"),
    )
    .unwrap();
    let revision = "ce8164ba92995cfa86005b6259115e08c8244253";
    let license_dir = upstream.join(revision);
    fs::create_dir_all(&license_dir).unwrap();
    fs::copy(
        project.join(format!("licenses/upstream/{revision}/LICENSE-APACHE")),
        license_dir.join("LICENSE-APACHE"),
    )
    .unwrap();

    let destination = root.join("dist/package");
    let result = assemble(&root, &destination, &metadata, target, &"f".repeat(40));
    assert!(result.is_ok(), "{}", result.unwrap_err());
    for (name, version, _) in dependencies {
        let license =
            fs::read(destination.join(format!("licenses/{name}-{version}/LICENSE-APACHE")))
                .unwrap();
        assert_eq!(
            license,
            fs::read(project.join(format!("licenses/upstream/{revision}/LICENSE-APACHE"))).unwrap()
        );
    }
    fs::remove_dir_all(root).unwrap();
}
