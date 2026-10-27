#[test]
fn default_application_build_uses_the_native_rust_ui() {
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let features = manifest
        .split_once("[features]")
        .unwrap()
        .1
        .split_once("[[bin]]")
        .unwrap()
        .0;
    assert!(
        features
            .lines()
            .any(|line| line.trim() == r#"default = ["native-ui-probe"]"#),
        "default application builds must use the native Rust UI"
    );
}
