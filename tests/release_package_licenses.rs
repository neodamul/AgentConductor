#[path = "support/release_package.rs"]
mod release_package_support;
use agentconductor::release_package::assemble;
use release_package_support::{fixture, temp};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

#[test]
fn requires_exact_upstream_license_revision_and_digest() {
    let root = temp();
    let target = "x86_64-pc-windows-msvc";
    let mut metadata = fixture(&root, target);
    let package = metadata["packages"][0].as_object_mut().unwrap();
    package.insert("name".into(), json!("webview2-com"));
    package.insert("version".into(), json!("0.39.1"));
    let source = root.join("dependency");
    fs::remove_file(source.join("LICENSE-MIT")).unwrap();
    let revision = "edc2caf886175ccaebe86078c9cfe1ae2a187328";
    fs::write(
        source.join(".cargo_vcs_info.json"),
        json!({"git":{"sha1":revision}}).to_string(),
    )
    .unwrap();
    let licenses = root.join("licenses/upstream");
    let license_dir = licenses.join(revision);
    fs::create_dir_all(&license_dir).unwrap();
    let license = b"Reviewed upstream license text\n";
    fs::write(license_dir.join("LICENSE"), license).unwrap();
    let sha = format!("{:x}", Sha256::digest(license));
    let provenance = json!([{"name":"webview2-com","version":"0.39.1","license":"MIT","vcs_commit":revision,"file":format!("{revision}/LICENSE"),"sha256":sha}]);
    fs::write(
        licenses.join("sources.json"),
        serde_json::to_vec(&provenance).unwrap(),
    )
    .unwrap();
    let destination = root.join("dist/package");
    assemble(&root, &destination, &metadata, target, &"b".repeat(40)).unwrap();
    assert_eq!(
        fs::read(destination.join("licenses/webview2-com-0.39.1/LICENSE")).unwrap(),
        license
    );
    fs::remove_dir_all(&destination).unwrap();
    fs::write(
        source.join(".cargo_vcs_info.json"),
        json!({"git":{"sha1":"b".repeat(40)}}).to_string(),
    )
    .unwrap();
    assert!(assemble(&root, &destination, &metadata, target, &"b".repeat(40)).is_err());
    assert!(!destination.exists());
    fs::write(
        source.join(".cargo_vcs_info.json"),
        json!({"git":{"sha1":revision}}).to_string(),
    )
    .unwrap();
    metadata["packages"][0]["version"] = json!("0.39.2");
    assert!(assemble(&root, &destination, &metadata, target, &"b".repeat(40)).is_err());
    assert!(!destination.exists());
    metadata["packages"][0]["version"] = json!("0.39.1");
    fs::write(license_dir.join("LICENSE"), b"altered\n").unwrap();
    assert!(assemble(&root, &destination, &metadata, target, &"b".repeat(40)).is_err());
    assert!(!destination.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn packages_reviewed_permissive_native_ui_dependency_licenses() {
    let root = temp();
    let target = "x86_64-pc-windows-msvc";
    let mut metadata = fixture(&root, target);
    let package = metadata["packages"][0].as_object_mut().unwrap();
    package.insert("name".into(), json!("clipboard-win"));
    package.insert("version".into(), json!("5.4.1"));
    package.insert("license".into(), json!("BSL-1.0"));
    let isc_source = root.join("isc-dependency");
    fs::create_dir_all(&isc_source).unwrap();
    fs::write(
        isc_source.join("Cargo.toml"),
        "[package]\nname = \"libloading\"\n",
    )
    .unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("licenses/upstream/f4ec9e702de2d0778bccff8525dc44e4cacac2d1/LICENSE"),
        isc_source.join("LICENSE"),
    )
    .unwrap();
    metadata["resolve"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"isc-dependency-id"}));
    metadata["packages"].as_array_mut().unwrap().push(json!({
        "id":"isc-dependency-id",
        "source":"registry+fixture",
        "name":"libloading",
        "version":"0.8.9",
        "license":"ISC",
        "manifest_path":isc_source.join("Cargo.toml")
    }));
    let revision = "3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342";
    let source = root.join("dependency");
    fs::remove_file(source.join("LICENSE-MIT")).unwrap();
    fs::write(
        source.join(".cargo_vcs_info.json"),
        json!({"git":{"sha1":revision}}).to_string(),
    )
    .unwrap();
    let licenses = root.join("licenses/upstream");
    fs::create_dir_all(licenses.join(revision)).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("licenses/upstream/sources.json"),
        licenses.join("sources.json"),
    )
    .unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("licenses/upstream/{revision}/LICENSE")),
        licenses.join(revision).join("LICENSE"),
    )
    .unwrap();

    let destination = root.join("dist/package");
    assemble(&root, &destination, &metadata, target, &"c".repeat(40)).unwrap();
    let bundled =
        fs::read_to_string(destination.join("licenses/clipboard-win-5.4.1/LICENSE")).unwrap();
    assert!(bundled.starts_with("Boost Software License - Version 1.0"));
    let bundled =
        fs::read_to_string(destination.join("licenses/libloading-0.8.9/LICENSE")).unwrap();
    assert!(bundled.contains("Permission to use, copy, modify, and/or distribute"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn includes_the_bundled_sqlite_copyright_disclaimer() {
    let root = temp();
    let target = "x86_64-pc-windows-msvc";
    let mut metadata = fixture(&root, target);
    metadata["packages"][0]["name"] = json!("libsqlite3-sys");
    let source = root.join("dependency/sqlite3");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("sqlite3.c"),
        b"/* SQLite author disclaims copyright to this source code. */\n",
    )
    .unwrap();
    let destination = root.join("dist/package");
    assemble(&root, &destination, &metadata, target, &"d".repeat(40)).unwrap();
    let notice = fs::read_to_string(
        destination.join("licenses/libsqlite3-sys-1.0.0/SQLITE-SOURCE-NOTICE.txt"),
    )
    .unwrap();
    assert!(notice.contains("author disclaims copyright"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_unreviewed_license_before_creating_package() {
    let root = temp();
    let target = "x86_64-pc-windows-msvc";
    let mut metadata = fixture(&root, target);
    metadata["packages"][0]["license"] = json!("AGPL-3.0-only");
    let destination = root.join("dist/package");
    assert!(assemble(&root, &destination, &metadata, target, &"c".repeat(40)).is_err());
    assert!(assemble(&root, &destination, &metadata, target, &"C".repeat(40)).is_err());
    assert!(assemble(&root, &destination, &metadata, "linux-gnu", &"c".repeat(40)).is_err());
    assert!(!destination.exists());
    fs::remove_dir_all(root).unwrap();

    let root = temp();
    let metadata = fixture(&root, target);
    fs::remove_file(root.join("dependency/LICENSE-MIT")).unwrap();
    let destination = root.join("dist/package");
    assert!(assemble(&root, &destination, &metadata, target, &"c".repeat(40)).is_err());
    assert!(!destination.exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reviewed_upstream_license_bytes_survive_git_autocrlf_checkout() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let relative = Path::new("licenses/upstream/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE");
    let original = fs::read(project.join(relative)).unwrap();
    let root = temp();
    let license = root.join(relative);
    fs::create_dir_all(license.parent().unwrap()).unwrap();
    fs::write(&license, &original).unwrap();
    fs::copy(project.join(".gitattributes"), root.join(".gitattributes")).unwrap();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "--quiet"]);
    git(&["-c", "core.autocrlf=false", "add", "."]);
    fs::remove_file(&license).unwrap();
    git(&[
        "-c",
        "core.autocrlf=true",
        "checkout",
        "--",
        relative.to_str().unwrap(),
    ]);
    assert_eq!(fs::read(&license).unwrap(), original);
    fs::remove_dir_all(root).unwrap();
}
