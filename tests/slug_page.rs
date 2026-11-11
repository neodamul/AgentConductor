//! Increment 6b: the workspace page shows slugs, progress, next phase and plan rules.
#[path = "support/catalog.rs"]
mod catalog;
const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

#[test]
fn page_offers_slug_progress_next_phase_and_rules() {
    // The first step's panel (its tab caption is the short "Workspace" since GitHub #432).
    let step1 = &HTML[HTML.find("id=\"start-panel-workspace\"").unwrap()
        ..HTML.find("id=\"start-panel-team\"").unwrap()];
    // R-74 / DD-82: the slug is named automatically; the page never asks for it.
    assert!(!step1.contains("id=\"start-slug\""), "no slug input");
    assert!(!SCRIPT.contains("q('slug')"), "the page sends no slug");
    // GitHub #396: the goals are a slide-over dialog opened from the counted Goal list button.
    assert!(HTML.contains("<dialog id=\"start-goals\" class=\"slide-panel\""));
    assert!(HTML.contains("id=\"start-goal-list-open\""));
    assert!(HTML.contains("<dialog id=\"start-rules\""));
    for needle in ["kind:'next_phase'", "kind:'save_plan_rules'"] {
        assert!(SCRIPT.contains(needle), "missing {needle}");
    }
    for text in ["Plan next phase", "Plan rules…"] {
        assert!(
            catalog::shows(SCRIPT, text)
                || catalog::shows(HTML, text)
                || (text == "Plan rules…"
                    && include_str!("../src/ui/app_menu.js").contains("'startup.plan_rules'")),
            "missing {text}"
        );
    }
}
