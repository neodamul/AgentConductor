//! Increment 6b: the workspace page shows slugs, progress, next phase and plan rules.
const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

#[test]
fn page_offers_slug_progress_next_phase_and_rules() {
    let step1 = &HTML
        [HTML.find("Workspace settings</h2>").unwrap()..HTML.find("Agent roles</h2>").unwrap()];
    // R-74 / DD-82: the slug is named automatically; the page never asks for it.
    assert!(!step1.contains("id=\"start-slug\""), "no slug input");
    assert!(!SCRIPT.contains("q('slug')"), "the page sends no slug");
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
    ] {
        assert!(SCRIPT.contains(needle), "missing {needle}");
    }
}
