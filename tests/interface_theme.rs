//! R-128 / DD-137 / QAS-124: light and dark themes from semantic tokens, chosen in Settings.
//! HOME is set once in this test binary so the preference stays inside the fixture.
use agentconductor::ui::Room;
use std::collections::BTreeSet;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const ROOM_CSS: &str = include_str!("../src/ui/room.css");
const MCP_CSS: &str = include_str!("../src/ui/mcp.css");
const STARTUP_CSS: &str = include_str!("../src/ui/startup.css");
const HTML: &str = include_str!("../src/ui/startup.html");
const STARTUP_JS: &str = include_str!("../src/ui/startup.js");

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
    for (name, css) in [
        ("room.css", rest.as_str()),
        ("startup.css", STARTUP_CSS),
        ("mcp.css", MCP_CSS),
    ] {
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
fn theme_is_saved_embedded_and_validated() {
    let (root, _plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let call = |room: &Room, v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
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
    let language =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert!(language.contains("\"theme\": \"dark\""), "{language}");
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

fn property<'a>(rule: &'a str, name: &str) -> Option<&'a str> {
    rule.split(';').find_map(|d| {
        let (key, value) = d.trim().split_once(':')?;
        (key == name).then_some(value)
    })
}

fn rgb(theme: &str, value: &str) -> [f64; 3] {
    let token = value
        .strip_prefix("var(")
        .unwrap()
        .strip_suffix(')')
        .unwrap();
    let hex = property(block(ROOM_CSS, theme), token)
        .unwrap()
        .strip_prefix('#')
        .unwrap();
    assert_eq!(hex.len(), 6);
    std::array::from_fn(|i| u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).unwrap() as f64 / 255.0)
}

fn contrast(theme: &str, text: &str, background: &str, surface: &str, opacity: f64) -> f64 {
    let parent = rgb(theme, surface);
    let luminance = |color: [f64; 3]| {
        [0.2126, 0.7152, 0.0722]
            .iter()
            .enumerate()
            .map(|(i, weight)| {
                let v = color[i] * opacity + parent[i] * (1.0 - opacity);
                weight
                    * if v <= 0.04045 {
                        v / 12.92
                    } else {
                        ((v + 0.055) / 1.055).powf(2.4)
                    }
            })
            .sum::<f64>()
    };
    let (a, b) = (
        luminance(rgb(theme, text)),
        luminance(rgb(theme, background)),
    );
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[test]
fn notices_have_readable_contrast_in_both_themes() {
    let normal = block(ROOM_CSS, ".notice");
    for theme in [":root", ":root[data-theme=\"dark\"]"] {
        for (label, rule) in [
            ("normal", normal),
            ("error", block(ROOM_CSS, ".notice.error")),
        ] {
            let text = property(rule, "color")
                .or_else(|| property(normal, "color"))
                .unwrap();
            let background = property(rule, "background")
                .or_else(|| property(normal, "background"))
                .unwrap();
            let ratio = contrast(theme, text, background, "var(--surface)", 1.0);
            assert!(ratio >= 4.5, "{theme} {label} notice contrast {ratio:.2}:1");
        }
    }
}

#[test]
fn primary_button_text_stays_readable_when_disabled() {
    let primary = block(ROOM_CSS, "button.primary");
    let disabled = if ROOM_CSS.contains("button.primary:disabled{") {
        block(ROOM_CSS, "button.primary:disabled")
    } else {
        ""
    };
    let alpha = property(disabled, "opacity")
        .or_else(|| property(block(ROOM_CSS, "button:disabled"), "opacity"))
        .map(|v| v.parse().unwrap())
        .unwrap_or(1.0);
    for theme in [":root", ":root[data-theme=\"dark\"]"] {
        for (label, rule, opacity) in [("active", primary, 1.0), ("disabled", disabled, alpha)] {
            let text = property(rule, "color")
                .or_else(|| property(primary, "color"))
                .unwrap();
            let background = property(rule, "background")
                .or_else(|| property(primary, "background"))
                .unwrap();
            for surface in ["var(--surface)", "var(--surface-2)"] {
                let ratio = contrast(theme, text, background, surface, opacity);
                assert!(
                    ratio >= 4.5,
                    "{theme} {label} primary contrast {ratio:.2}:1 on {surface}"
                );
            }
        }
    }
}
