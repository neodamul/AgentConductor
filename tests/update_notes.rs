use std::path::Path;

#[test]
fn remote_release_notes_are_literal_scrollable_text() {
    assert!(
        !Path::new("tests/test_update_notes.py").exists(),
        "the Python contract must be retired after its Rust replacement exists"
    );
    let script = include_str!("../src/ui/startup.js");
    let render = script
        .split("function renderUpdate(){")
        .nth(1)
        .unwrap()
        .split("  async function updateRequest")
        .next()
        .unwrap();
    assert!(render.contains("q('update-body').hidden=!u.notes;"));
    assert!(render.contains("AC_I18N.bind(q('update-body'),u.notes||'');"));
    assert!(
        !render.contains("innerHTML"),
        "release text must not execute markup"
    );
    let i18n = include_str!("../src/ui/i18n.js");
    assert!(i18n.contains("node.textContent=text"));
    assert!(!i18n.contains("node.innerHTML"));

    let html = include_str!("../src/ui/startup.html");
    assert!(html.contains("id=\"start-update-body\""));
    assert!(html.contains("id=\"start-update-notes\""));
    assert!(
        html.find("id=\"start-update-notes\"").unwrap()
            < html.find("id=\"start-update-body\"").unwrap()
    );

    let css = include_str!("../src/ui/startup.css");
    let rule = css
        .split("#start-update-body{")
        .nth(1)
        .unwrap()
        .split('}')
        .next()
        .unwrap();
    assert!(rule.contains("overflow:auto"));
    assert!(rule.contains("max-height:"));
    assert!(rule.contains("white-space:pre-wrap"));
}

#[test]
fn available_update_uses_a_downward_arrow() {
    let html = include_str!("../src/ui/startup.html");
    let button = html
        .split("id=\"start-update-button\"")
        .nth(1)
        .unwrap()
        .split("</button>")
        .next()
        .unwrap();
    assert!(button.ends_with('⬇'), "{button}");
    assert!(!button.contains('⬆'));
}
