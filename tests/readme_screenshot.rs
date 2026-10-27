const README: &str = include_str!("../README.md");

#[test]
fn readme_uses_only_the_current_verified_workspace_capture() {
    assert_eq!(README.matches("![").count(), 1);
    assert!(
        README
            .contains("![Workspace and goal configuration](docs/assets/ui-workspace-current.png)")
    );
    assert!(!include_bytes!("../docs/assets/ui-workspace-current.png").is_empty());
}
