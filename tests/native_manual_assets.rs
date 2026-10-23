#[test]
fn manual_screenshot_status_uses_current_rust_assets_without_acceptance_credit() {
    let plan = include_str!("../docs/07-implementation-plan.md");
    let english = include_str!("../docs/user-manual.md");
    let korean = include_str!("../docs/user-manual.ko.md");
    let wiki = include_str!("../knowledge/wiki/native-rust-ui-migration.md");
    let evidence =
        include_str!("../docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json");
    assert!(english.contains("assets/ui-agent-room-native.png"));
    assert!(korean.contains("assets/ui-agent-room-native-ko.png"));
    assert!(plan.contains(
        "Task 2.22 / Manual assets: refreshed bilingual screenshots from the current Rust UI"
    ));
    assert!(plan.contains(
        "exact-package element-by-element comparison against the design guide remains open"
    ));
    assert!(!plan.contains(
        "screenshot references embedded in the manual are v0.1.21/v0.1.24 legacy assets"
    ));
    assert!(evidence.contains("\"packaged_workflow_acceptance\": \"15/25\""));
    assert!(evidence.contains("\"acceptance_credit\": 0"));
    assert!(wiki.contains(
        "bilingual Agent Room manual images are now retained source-built Rust UI captures"
    ));
    assert!(!wiki.contains(
        "The separate embedded screenshot is stale and still needs a current, safe capture."
    ));
}
