//! Increment 6b: the workspace page shows slugs, progress, next phase and plan rules.
#[path = "support/catalog.rs"]
mod catalog;
const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

#[test]
fn page_offers_slug_progress_next_phase_and_rules() {
    let at = |text: &str| {
        HTML.find(&format!("\"{}\"", catalog::id(text)))
            .unwrap_or_else(|| panic!("{text}"))
    };
    let settings = at("Workspace settings");
    let step1 = &HTML[settings..at("Agent roles")];
    // R-74 / DD-82: the slug is named automatically; the page never asks for it.
    assert!(!step1.contains("id=\"start-slug\""), "no slug input");
    assert!(!SCRIPT.contains("q('slug')"), "the page sends no slug");
    let goals = HTML
        .find("id=\"start-goals\"")
        .expect("goals and progress card");
    assert!(goals < settings, "progress overview comes first");
    assert!(HTML.contains("<dialog id=\"start-rules\""));
    for needle in ["kind:'next_phase'", "kind:'save_plan_rules'"] {
        assert!(SCRIPT.contains(needle), "missing {needle}");
    }
    for text in ["Plan next phase", "Plan rules…"] {
        assert!(
            catalog::shows(SCRIPT, text) || catalog::shows(HTML, text),
            "missing {text}"
        );
    }
}
