//! GitHub #406 / DD-222: the team briefing, the coordinator chat and Settings open from the right
//! over the unchanged content, one at a time, like the Goal list.
const ROOM_HTML: &str = include_str!("../src/ui/room.html");
const START_HTML: &str = include_str!("../src/ui/startup.html");
const START_CSS: &str = include_str!("../src/ui/startup.css");
const CHAT_CSS: &str = include_str!("../src/ui/chat.css");
const CHAT_JS: &str = include_str!("../src/ui/chat.js");

#[test]
fn briefing_and_settings_use_the_slide_over_panel() {
    assert!(ROOM_HTML.contains("<dialog id=\"briefing-dialog\" class=\"slide-panel\""));
    assert!(START_HTML.contains("<dialog id=\"start-settings\" class=\"slide-panel\""));
    // The id rule that caps the old centered dialog must not shorten the full-height panel.
    assert!(START_CSS.contains("#start-settings.slide-panel{max-height:100vh}"));
}

#[test]
fn the_chat_panel_slides_in_and_yields_to_any_other_slide_over() {
    assert!(CHAT_CSS.contains(".chat-panel:not([hidden]){animation:slide-in .18s ease-out}"));
    assert!(
        CHAT_CSS
            .contains("prefers-reduced-motion:reduce){.chat-panel:not([hidden]){animation:none}")
    );
    // Opening a slide-over dialog closes the chat so only one panel is active at a time.
    assert!(CHAT_JS.contains("dialog.slide-panel[open]"));
    assert!(CHAT_JS.contains("attributeFilter:['open']"));
}
