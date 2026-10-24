//! Current bilingual Workspace screenshots must stay paired with the manuals.

#[test]
fn korean_workspace_screenshots_are_current_and_embedded_in_the_manual() {
    let english_manual = include_str!("../docs/user-manual.md");
    let english_html = include_str!("../docs/user-manual.html");
    let manual = include_str!("../docs/user-manual.ko.md");
    let html = include_str!("../docs/user-manual.ko.html");
    let provenance = include_str!("../knowledge/wiki/native-screenshot-provenance.md");
    let manual_wiki = include_str!("../knowledge/wiki/offline-html-manual.md");

    assert!(manual.contains("assets/ui-workspace-korean-current.png"));
    assert!(manual.contains("assets/ui-workspace-dark-current.png"));
    assert!(english_manual.contains("assets/ui-agent-room-native.png"));
    assert!(
        manual_wiki
            .contains("The current public manuals use PNG captures for Workspace and Agent Room")
    );
    assert!(!manual_wiki.contains("the public manuals use Rust-native `.jpg` captures"));
    assert!(manual.contains("assets/ui-agent-room-native-ko.png"));
    assert!(include_bytes!("../docs/assets/ui-workspace-korean-current.png").len() > 100_000);
    assert!(include_bytes!("../docs/assets/ui-workspace-dark-current.png").len() > 100_000);
    assert_eq!(
        html.matches("data:image/png;base64").count(),
        manual.matches("![").count()
    );
    assert_eq!(
        english_html.matches("data:image/png;base64").count(),
        english_manual.matches("![").count()
    );
    let docs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
    assert_eq!(
        agentconductor::manual::render(manual, &docs)
            .unwrap()
            .replace("\r\n", "\n"),
        html.replace("\r\n", "\n"),
        "the offline HTML manual must embed the current Markdown assets"
    );
    assert_eq!(
        agentconductor::manual::render(english_manual, &docs)
            .unwrap()
            .replace("\r\n", "\n"),
        english_html.replace("\r\n", "\n"),
        "the English offline HTML manual must embed its language-matched image"
    );
    assert!(
        provenance
            .contains("Korean light, and Korean dark Workspace captures are current source-build")
    );
}
