//! GitHub #408 / visual-spec: step-card action buttons have a leading 13px icon mapped to the action.
const JS: &str = include_str!("../src/ui/action_icons.js");
const HTML: &str = include_str!("../src/ui/startup.html");
const MCP_HTML: &str = include_str!("../src/ui/mcp.html");
const ROOM_HTML: &str = include_str!("../src/ui/room.html");
const CSS: &str = include_str!("../src/ui/startup.css");

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
                || MCP_HTML.contains(&format!("id=\"{id}\"")),
            "{id} is a real button"
        );
        icons.insert(icon.to_string());
    }
    assert!(icons.len() >= 5);
    for icon in icons {
        assert!(
            CSS.contains(&format!("[data-action-icon=\"{icon}\"]{{--ai:url(")),
            "{icon}"
        );
    }
    assert!(CSS.contains(
        "[data-action-icon]{display:flex;align-items:center;gap:6px}[data-action-icon]::before{content:\"\";display:block;flex:none;width:13px;height:13px;"
    ));
}
