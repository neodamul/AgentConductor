use agentconductor::release_package::assemble;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

fn fixture(root: &Path, target: &str) -> Value {
    fs::create_dir_all(root.join("docs/assets")).unwrap();
    fs::write(
        root.join("README.md"),
        "# Product\n![UI](docs/assets/live.jpg)\n",
    )
    .unwrap();
    fs::write(root.join("docs/assets/live.jpg"), b"live").unwrap();
    fs::write(root.join("docs/assets/mock.jpg"), b"unused").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nversion = \"0.1.0\"\n").unwrap();
    fs::write(root.join("Cargo.lock"), "locked fixture\n").unwrap();
    fs::write(
        root.join("LICENSE"),
        "MIT License\n\nCopyright (c) fixture\n",
    )
    .unwrap();
    let icon = root.join("src/ui/assets/ac.icns");
    fs::create_dir_all(icon.parent().unwrap()).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/assets/ac.icns"),
        icon,
    )
    .unwrap();
    let name = if target.contains("windows") {
        "agentconductor.exe"
    } else {
        "agentconductor"
    };
    let binary = root.join("target/release").join(name);
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, b"exact native binary").unwrap();
    let source = root.join("dependency");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("Cargo.toml"), "[package]\nname = \"dep\"\n").unwrap();
    fs::write(source.join("LICENSE-MIT"), "MIT fixture license\n").unwrap();
    json!({
        "resolve":{"nodes":[{"id":"dependency-id"}]},
        "packages":[{"id":"dependency-id","source":"registry+fixture","name":"dep","version":"1.0.0","license":"MIT","manifest_path":source.join("Cargo.toml")}]
    })
}

fn temp() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "ac-release-package-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn assembles_macos_and_windows_packages_with_reviewed_dependencies() {
    for target in ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"] {
        let root = temp();
        let metadata = fixture(&root, target);
        let destination = root.join("dist/package");
        let manifest = assemble(&root, &destination, &metadata, target, &"a".repeat(40)).unwrap();
        let name = if target.contains("windows") {
            "agentconductor.exe"
        } else {
            "agentconductor"
        };
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
        let prerequisite = if target.contains("windows") {
            "Microsoft Edge WebView2 Runtime"
        } else {
            "system WebKit"
        };
        assert!(
            manifest["external_requirements"]
                .as_array()
                .unwrap()
                .iter()
                .any(|value| value == prerequisite)
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
