use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub fn fixture(root: &Path, target: &str) -> Value {
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

pub fn temp() -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "ac-release-package-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
