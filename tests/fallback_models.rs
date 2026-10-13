//! R-71 / DD-77 / QAS-64: the fallback model list comes from the fallback agent's catalog,
//! fetched on demand, and the dialog explains what the fallback is.
#[path = "support/catalog.rs"]
mod catalog;
const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

fn function(name: &str) -> &'static str {
    let body = &SCRIPT[SCRIPT.find(&format!("function {name}(")).unwrap()..];
    &body[..body.find('\n').unwrap()]
}

#[test]
fn choosing_a_fallback_agent_fetches_and_shows_its_models() {
    let models = function("fallbackModels");
    assert!(
        models.contains("addedAgents.add("),
        "missing catalog is queued"
    );
    assert!(catalog::shows(models, "Loading models…"));
    assert!(catalog::shows(models, "Choose an alternate agent first"));
    assert!(models.contains("disabled=!id"));
    // An open dialog picks up the list when it arrives, without moving focus.
    assert!(SCRIPT.contains(
        "if(q('agent-dialog').open&&document.activeElement!==q('fallback-model'))fallbackModels("
    ));
}

#[test]
fn dialog_explains_the_fallback() {
    let dialog = &HTML[HTML.find("<dialog id=\"start-agent-dialog\"").unwrap()..];
    let dialog = &dialog[..dialog.find("</dialog>").unwrap()];
    assert!(catalog::shows(dialog, "Alternate agent's model"));
    assert!(dialog.contains("startup.when_this_agent_s_account_is_limited_its"));
}

#[test]
fn settings_button_is_an_icon_with_an_accessible_name() {
    let titlebar = &HTML[..HTML.find("</header>").unwrap()];
    let button = &titlebar[titlebar.find("id=\"start-settings-button\"").unwrap()..];
    let button = &button[..button.find("</button>").unwrap()];
    let settings = catalog::id("Settings");
    assert!(
        button.contains(&format!("data-i18n-aria-label=\"{settings}\""))
            && button.contains(&format!("data-i18n-title=\"{settings}\""))
    );
    let css = include_str!("../src/ui/startup.css");
    assert!(css.contains("width:32px;height:32px"));
    assert!(css.contains("font-size:26px"));
    assert!(button.ends_with(">⚙"), "icon only: {button}");
}
