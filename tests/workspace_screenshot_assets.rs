//! Current bilingual Workspace screenshots must stay paired with the manuals.

#[test]
fn korean_workspace_screenshots_are_current_and_embedded_in_the_manual() {
    let manual = include_str!("../docs/user-manual.ko.md");
    let html = include_str!("../docs/user-manual.ko.html");
    let provenance = include_str!("../knowledge/wiki/native-screenshot-provenance.md");

    assert!(manual.contains("assets/ui-workspace-korean-current.png"));
    assert!(manual.contains("assets/ui-workspace-dark-current.png"));
    assert!(include_bytes!("../docs/assets/ui-workspace-korean-current.png").len() > 100_000);
    assert!(include_bytes!("../docs/assets/ui-workspace-dark-current.png").len() > 100_000);
    assert_eq!(html.matches("data:image/png;base64").count(), 3);
    let docs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    assert_eq!(
        agentconductor::manual::render(manual, &docs).unwrap(),
        html,
        "the offline HTML manual must embed the current Markdown assets"
    );
    assert!(
        provenance
            .contains("Korean light, and Korean dark Workspace captures are current source-build")
    );
}
