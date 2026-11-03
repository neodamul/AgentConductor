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

#[test]
fn default_executable_routes_empty_arguments_to_the_workspace_window() {
    let source =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs")).unwrap();
    assert!(
        source.contains("desktop::is_workspace_launch(&args)")
            && source.contains("desktop::launch_default()"),
        "the shipped agentconductor binary must open the Rust Workspace UI with no arguments"
    );
}
