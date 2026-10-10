//! DD-39 / QAS-26: the workspace offers a side-effect-free way back.
const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

fn function_body(name: &str) -> &'static str {
    let start = SCRIPT
        .find(&format!("function {name}("))
        .unwrap_or_else(|| panic!("missing function {name}"));
    let body = &SCRIPT[start..];
    let mut depth = 0;
    for (index, ch) in body.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &body[..=index];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function {name}");
}

#[test]
fn titlebar_offers_back_to_run_and_discard_with_a_confirmation_dialog() {
    let titlebar = &HTML[..HTML.find("</header>").unwrap()];
    assert!(titlebar.contains("id=\"start-back\""));
    assert!(titlebar.contains("id=\"start-discard\""));
    assert!(HTML.contains("<dialog id=\"start-leave\""));
}

#[test]
fn discarding_rebuilds_the_saved_view_without_any_native_request() {
    let discard = function_body("discardDraft");
    for forbidden in ["request(", "action(", "save(", "kind:"] {
        assert!(
            !discard.contains(forbidden),
            "discard must stay view-only: {forbidden}"
        );
    }
    assert!(discard.contains("state.config"));
    assert!(discard.contains("dirty=false"));
    assert!(discard.contains("planDirty=false"));
}
