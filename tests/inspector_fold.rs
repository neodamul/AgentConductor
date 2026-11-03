//! GitHub #146 / DD-175, amended by #394 / R-210: the docked task detail column starts closed and is toggled from the titlebar.
use std::process::Command;

const HTML: &str = include_str!("../src/ui/room.html");
const CSS: &str = include_str!("../src/ui/room.css");
const JS: &str = include_str!("../src/ui/room.js");

#[test]
fn the_fold_logic_follows_the_decision_rules() {
    let output = Command::new("node")
        .arg("tests/inspector_fold_runtime.cjs")
        .output()
        .expect("node runs the runtime test");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn the_page_has_a_titlebar_toggle_and_a_quiet_blinking_dot() {
    let startup = include_str!("../src/ui/startup.html");
    for page in [HTML, startup] {
        assert!(page.contains("class=\"quiet small-button inspector-switch\""));
        assert!(page.contains("aria-expanded=\"false\""));
        assert!(page.contains("class=\"inspector-alert\" role=\"status\""));
    }
    assert!(HTML.contains("id=\"inspector-toggle\""));
    assert!(startup.contains("id=\"start-inspector-toggle\""));
    assert!(
        !HTML.contains("inspector-heading><h2><span data-i18n=\"room.task_inspector\"></span></h2><span class=\"inspector-dot\"></span><span id=\"inspector-alert"),
        "no toggle or alert inside the inspector"
    );
    assert!(CSS.contains(".insp-closed"));
    assert!(CSS.contains("@keyframes inspector-blink"));
    assert!(CSS.contains("prefers-reduced-motion"));
    let dot = CSS
        .split(".inspector-alert{")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(dot.contains("var(--warn)"), "{dot}");
    for id in [
        "room.inspector_fold",
        "room.inspector_unfold",
        "room.inspector_decision",
    ] {
        for catalog in [
            include_str!("../src/ui/messages/en.json"),
            include_str!("../src/ui/messages/ko.json"),
        ] {
            assert!(catalog.contains(&format!("\"{id}\"")), "{id}");
        }
    }
    assert!(JS.contains("window.acInspector"));
    let guide = include_str!("../docs/design-guide.md");
    assert!(guide.contains("reduced-motion") && guide.contains("decision"));
}
