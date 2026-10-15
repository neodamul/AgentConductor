use std::path::Path;

#[test]
fn repository_license_and_manifest_agree_on_mit() {
    let license = include_str!("../LICENSE");
    let manifest = include_str!("../Cargo.toml");
    let readme = include_str!("../README.md");
    assert!(license.starts_with("MIT License"));
    assert!(license.contains("Permission is hereby granted, free of charge"));
    assert!(manifest.contains("license = \"MIT\""));
    assert!(readme.contains("](LICENSE)"));
    assert!(!Path::new("tests/test_project_license.py").exists());
}
