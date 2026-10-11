//! Increment 6b: the workspace page shows slugs, progress, next phase and plan rules.
const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

#[test]
fn page_offers_slug_progress_next_phase_and_rules() {
    let step1 = &HTML
        [HTML.find("Workspace settings</h2>").unwrap()..HTML.find("Agent roles</h2>").unwrap()];
    assert!(step1.contains("id=\"start-slug\""), "slug field in step 1");
    let goals = HTML
        .find("id=\"start-goals\"")
        .expect("goals and progress card");
    assert!(
        goals < HTML.find("Workspace settings</h2>").unwrap(),
        "progress overview comes first"
    );
    assert!(HTML.contains("<dialog id=\"start-rules\""));
    for needle in [
        "kind:'next_phase'",
        "'Plan next phase'",
        "kind:'save_plan_rules'",
        "'Plan rules…'",
        "function suggestSlug(",
        "slug:",
    ] {
        assert!(SCRIPT.contains(needle), "missing {needle}");
    }
}
