//! GitHub #408 / visual-spec: step-card action buttons have a leading 13px icon mapped to the action.
const JS: &str = include_str!("../src/ui/action_icons.js");
const HTML: &str = include_str!("../src/ui/startup.html");
const MCP_HTML: &str = include_str!("../src/ui/mcp.html");
const OPTIMIZE_HTML: &str = include_str!("../src/ui/optimize.html");
const ROOM_HTML: &str = include_str!("../src/ui/room.html");
const CSS: &str = include_str!("../src/ui/startup.css");
const ROOM_LAYOUT_CSS: &str = include_str!("../src/ui/room_layout.css");

#[test]
fn every_mapped_action_button_exists_and_every_icon_has_a_mask() {
    let map = JS
        .split("const ICONS = {")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    let mut icons = std::collections::BTreeSet::new();
    for pair in map.split(',') {
        let mut parts = pair.split(':');
        let id = parts.next().unwrap().trim().trim_matches('\'');
        let icon = parts.next().unwrap().trim().trim_matches('\'');
        assert!(
            HTML.contains(&format!("id=\"{id}\""))
                || ROOM_HTML.contains(&format!("id=\"{id}\""))
                || MCP_HTML.contains(&format!("id=\"{id}\""))
                || OPTIMIZE_HTML.contains(&format!("id=\"{id}\"")),
            "{id} is a real button"
        );
        icons.insert(icon.to_string());
    }
    assert!(icons.len() >= 5);
    let icon_css = format!(
        "{CSS}{ROOM_LAYOUT_CSS}{}",
        include_str!("../src/ui/run_cards.css")
    );
    for icon in icons {
        assert!(
            icon_css.contains(&format!("[data-action-icon=\"{icon}\"]{{--ai:url(")),
            "{icon}"
        );
    }
    assert!(CSS.contains(
        "[data-action-icon]{display:flex;align-items:center;gap:6px}[data-action-icon]::before{content:\"\";display:block;flex:none;width:13px;height:13px;"
    ));
}

#[test]
fn run_action_buttons_use_shared_svg_icons() {
    for id in [
        "verify-stop",
        "verify-wait",
        "review-revise",
        "denied-reassign",
        "review-deny",
        "tool-decision-deny",
        "tool-decision-allow",
        "denied-resume",
    ] {
        assert!(
            JS.contains(&format!("'{id}':")),
            "{id} lacks an icon mapping"
        );
    }
    let run_css = include_str!("../src/ui/run_cards.css");
    for glyph in [
        "content:'□ '",
        "content:'→ '",
        "content:'× '",
        "content:'✓ '",
        "content:'▷ '",
    ] {
        assert!(!run_css.contains(glyph), "text glyph remains: {glyph}");
    }
}
