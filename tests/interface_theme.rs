#![cfg(unix)]
//! R-128 / DD-137 / QAS-124: light and dark themes from semantic tokens, chosen in Settings.
//! HOME is set once in this test binary so the preference stays inside the fixture.
use agentconductor::ui::Room;
use std::collections::BTreeSet;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const ROOM_CSS: &str = include_str!("../src/ui/room.css");
const STARTUP_CSS: &str = include_str!("../src/ui/startup.css");
const HTML: &str = include_str!("../src/ui/startup.html");
const STARTUP_JS: &str = include_str!("../src/ui/startup.js");
const ROOM_JS: &str = include_str!("../src/ui/room.js");

fn block<'a>(css: &'a str, selector: &str) -> &'a str {
    let start = css.find(&format!("{selector}{{")).expect(selector) + selector.len() + 1;
    &css[start..start + css[start..].find('}').unwrap()]
}

fn tokens(block: &str) -> BTreeSet<String> {
    block
        .split(';')
        .filter_map(|d| d.trim().strip_prefix("--"))
        .map(|d| d.split(':').next().unwrap().to_string())
        .collect()
}

#[test]
fn colors_come_only_from_the_light_and_dark_token_sets() {
    let light = block(ROOM_CSS, ":root");
    let dark = block(ROOM_CSS, ":root[data-theme=\"dark\"]");
    let (l, d) = (tokens(light), tokens(dark));
    assert!(l.len() >= 20, "{l:?}");
    for t in &d {
        assert!(l.contains(t), "dark-only token --{t}");
    }
    for t in [
        "bg", "surface", "ink", "soft", "line", "bar", "accent", "ok", "warn", "err",
    ] {
        assert!(d.contains(t), "dark set misses --{t}");
    }
    let rest = ROOM_CSS.replacen(light, "", 1).replacen(dark, "", 1);
    for (name, css) in [("room.css", rest.as_str()), ("startup.css", STARTUP_CSS)] {
        // Only declaration values count; `#id` selectors are not colors.
        let hex: Vec<String> = css
            .split('{')
            .skip(1)
            .flat_map(|rule| rule.split('}').next().unwrap().split(';'))
            .filter_map(|decl| decl.split_once(':').map(|(_, value)| value))
            .flat_map(|value| value.match_indices('#').map(move |(i, _)| &value[i..]))
            .filter(|v| {
                let n = v[1..].chars().take_while(|c| c.is_ascii_hexdigit()).count();
                matches!(n, 3 | 4 | 6 | 8)
            })
            .map(|v| v.chars().take(9).collect())
            .collect();
        assert!(hex.is_empty(), "{name} has color literals: {hex:?}");
        for named in ["white", "black"] {
            for prop in ["background:", "color:", "border-color:"] {
                assert!(
                    !css.contains(&format!("{prop}{named}")),
                    "{name}: {prop}{named}"
                );
            }
        }
    }
}

#[test]
fn settings_offer_light_and_dark() {
    let dialog = &HTML[HTML.find("<dialog id=\"start-settings\"").unwrap()..];
    let dialog = &dialog[..dialog.find("</dialog>").unwrap()];
    assert!(dialog.contains("<select id=\"start-theme\">"));
    assert!(
        dialog.contains("<option value=\"light\"") && dialog.contains("<option value=\"dark\"")
    );
    assert!(STARTUP_JS.contains("kind:'set_theme'"));
    assert!(STARTUP_JS.contains("document.documentElement.dataset.theme="));
}

#[test]
fn session_status_is_compact_and_never_leaves_the_rail() {
    let row = &ROOM_JS[ROOM_JS
        .find("const body=node('span',undefined,'session-body'),meta=")
        .unwrap()..];
    let row = &row[..row.find("row.prepend(button)").unwrap()];
    assert!(
        row.contains("'session-symbol'") && row.contains("session-status"),
        "icon and tag kept"
    );
    let rule = block(ROOM_CSS, ".session-choice .session-meta .session-status");
    assert!(
        rule.contains("max-width:100%") && rule.contains("white-space:normal"),
        "{rule}"
    );
    assert!(block(ROOM_CSS, ".session-choice .session-meta").contains("flex-wrap:wrap"));
}

#[test]
fn theme_is_saved_embedded_and_validated() {
    let (root, _plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let call = |room: &Room, v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    assert!(
        room.document().unwrap().contains("data-theme=\"light\""),
        "light by default"
    );
    assert!(
        call(
            &room,
            serde_json::json!({"kind":"set_theme","theme":"sepia"})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"set_theme","theme":"dark"}),
    )
    .unwrap();
    drop(room);
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    let page = room.document().unwrap();
    assert!(
        page.contains("data-theme=\"dark\""),
        "a new window keeps the theme"
    );
    let language =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert!(language.contains("\"theme\": \"dark\""), "{language}");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
