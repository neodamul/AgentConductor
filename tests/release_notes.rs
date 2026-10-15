use agentconductor::release_notes::section;

#[test]
fn extracts_only_the_requested_tag_section() {
    assert_eq!(
        section("## v1\n- first\n\n## v2\n- second\n", "v1").unwrap(),
        "- first\n"
    );
}

#[test]
fn rejects_missing_or_empty_sections() {
    assert!(section("## v1\n- note\n", "v2").is_err());
    assert!(section("## v1\n\n## v2\n- note\n", "v1").is_err());
}

#[test]
fn release_workflow_uses_the_rust_cli() {
    let workflow = include_str!("../.github/workflows/release.yml");
    assert!(workflow.contains("cargo run --release --locked -- release notes"));
    assert!(!workflow.contains("scripts/release_notes.py"));
}

#[test]
fn preserves_utf8_notes() {
    assert_eq!(
        section("## v1\n- usage ≈0.1 % → done\n", "v1").unwrap(),
        "- usage ≈0.1 % → done\n"
    );
}

#[test]
fn repository_retains_every_initial_preview_release_section() {
    let changelog = include_str!("../CHANGELOG.md");
    for tag in [
        "v0.1.0-preview.1",
        "v0.1.0-preview.2",
        "v0.1.0-preview.3",
        "v0.1.0-preview.4",
        "v0.1.0-preview.5",
        "v0.1.0-preview.6",
        "v0.1.0",
    ] {
        assert!(!section(changelog, tag).unwrap().trim().is_empty(), "{tag}");
    }
}
