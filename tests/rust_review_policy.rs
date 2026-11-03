#[test]
fn issue_169_policy_and_traceability_are_present_on_the_rust_migration_branch() {
    let policy = include_str!("../docs/13-rust-review-policy.md");
    let requirements = include_str!("../docs/01-requirements.md");
    let goals = include_str!("../docs/02-business-goals.md");
    let features = include_str!("../docs/03-system-features.md");
    let use_cases = include_str!("../docs/04-usecases.md");
    let analysis = include_str!("../docs/04b-functional-nonfunctional-analysis.md");
    let qas = include_str!("../docs/05-qas.md");
    let drivers = include_str!("../docs/05b-design-drivers.md");
    let decisions = include_str!("../docs/06-design-decisions.md");
    let docs_index = include_str!("../docs/README.md");
    let changelog = include_str!("../CHANGELOG.md");
    let plan = include_str!("../docs/07-implementation-plan.md");
    let wiki = include_str!("../knowledge/wiki/rust-review-policy.md");
    let wiki_index = include_str!("../knowledge/wiki/index.md");

    assert!(policy.contains("https://google.github.io/styleguide/rustguide.html"));
    assert!(policy.contains("## Review rules"));
    assert!(policy.contains("## Explicit exclusions"));
    assert!(policy.contains("## Porting"));
    assert!(policy.contains("Existing code is not swept"));
    for document in [
        requirements,
        goals,
        features,
        use_cases,
        analysis,
        qas,
        decisions,
    ] {
        assert!(document.contains("R-174 → BG-129 → SF-162 → UC-165"));
        assert!(document.contains("QAS-171 → DDR-34 → DD-185"));
    }
    assert!(drivers.contains("### Driver DDR-34 Review guidance instead of new gates"));
    assert!(drivers.contains("Trace: QAS-171."));
    assert!(docs_index.contains("R-174 Rust review policy from the Google Rust Style Guide"));
    assert!(docs_index.contains("13-rust-review-policy.md"));
    assert!(changelog.contains("GitHub #169"));
    assert!(plan.contains(
        "### Port the Google Rust review policy to the Rust migration branch — GitHub #169"
    ));
    assert!(plan.contains("port the policy and its R-174 → DD-185 architecture trace"));
    assert!(wiki.contains("R-174 → BG-129 → SF-162 → UC-165"));
    assert!(wiki_index.contains("[Rust review policy](rust-review-policy.md)"));
}
