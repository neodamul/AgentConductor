#![cfg(unix)]
//! R-70 / DD-76 / QAS-63: English by default, Korean on demand, UTF-8 throughout. HOME is
//! set once in this test binary so the preference stays inside the fixture.
use agentconductor::ui::Room;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");
const ROOM: &str = include_str!("../src/ui/room.html");
const I18N: &str = include_str!("../src/ui/i18n.js");
const KO: &str = include_str!("../src/ui/i18n_ko.js");

fn call(room: &Room, value: serde_json::Value) -> Result<serde_json::Value, String> {
    room.dispatch(value.to_string().as_bytes())
}

#[test]
fn settings_menu_offers_english_and_korean() {
    let titlebar = &HTML[..HTML.find("</header>").unwrap()];
    assert!(titlebar.contains("id=\"start-settings-button\""));
    let dialog = &HTML[HTML
        .find("<dialog id=\"start-settings\"")
        .expect("settings dialog")..];
    let dialog = &dialog[..dialog.find("</dialog>").unwrap()];
    assert!(dialog.contains("<option value=\"en\">English</option>"));
    assert!(dialog.contains("<option value=\"ko\">한국어</option>"));
    assert!(ROOM.contains("<meta charset=\"utf-8\">"));
}

#[test]
fn language_is_saved_embedded_and_validated() {
    let (root, _plan) = support::setup("shared");
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    let page = room.document().unwrap();
    assert!(page.contains("<html lang=\"en\">"), "English by default");
    assert!(
        page.contains("const AC_KO=") && page.contains("function acSwap("),
        "translation layer ships in the page"
    );
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["language"],
        "en"
    );

    assert!(
        call(
            &room,
            serde_json::json!({"kind":"set_language","language":"xx"})
        )
        .is_err()
    );
    call(
        &room,
        serde_json::json!({"kind":"set_language","language":"ko"}),
    )
    .unwrap();
    let stored =
        std::fs::read_to_string(root.join("home/.agentconductor/preferences.json")).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored).unwrap()["language"],
        "ko"
    );
    drop(room);
    // A new window starts in the saved language.
    let room = Room::startup(&root.join("desktop"), exe).unwrap();
    assert!(room.document().unwrap().contains("<html lang=\"ko\">"));
    assert_eq!(
        call(&room, serde_json::json!({"kind":"snapshot"})).unwrap()["language"],
        "ko"
    );
    call(
        &room,
        serde_json::json!({"kind":"set_language","language":"en"}),
    )
    .unwrap();
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn exact_strings_and_prefixes_translate_and_invert() {
    let swap = &I18N[I18N.find("function acSwap(").unwrap()..];
    let swap = &swap[..swap.find('\n').unwrap()];
    let invert = &I18N[I18N.find("function acInvert(").unwrap()..];
    let invert = &invert[..invert.find('\n').unwrap()];
    let script = format!(
        "{KO}\n{swap}\n{invert}\nconst a=require('node:assert/strict');\
         a.equal(acSwap('  Workspace settings ',AC_KO),'  워크스페이스 설정 ');\
         a.equal(acSwap('Not in the table',AC_KO),'Not in the table');\
         a.equal(acSwap('Local workspace · /tmp/a',AC_KO),'로컬 워크스페이스 · /tmp/a');\
         const back=acInvert(AC_KO);a.equal(acSwap('워크스페이스 설정',back),'Workspace settings');\
         a.equal(acSwap('로컬 워크스페이스 · /tmp/a',back),'Local workspace · /tmp/a');"
    );
    let status = std::process::Command::new("node")
        .args(["-e", &script])
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn user_content_is_never_translated_and_english_costs_nothing() {
    assert!(I18N.contains("if(node.tagName==='TEXTAREA')attrs(node,dict)"), "placeholder translated, text kept");
    for skipped in ["TEXTAREA", "PRE", "CODE"] {
        assert!(I18N.contains(&format!("'{skipped}'")), "{skipped} skipped");
    }
    // The observer only runs while a non-English language is active.
    assert!(I18N.contains("observer.disconnect()"));
    // Every Korean entry translates a string that exists in the interface sources.
    let sources = [
        HTML,
        ROOM,
        include_str!("../src/ui/startup.js"),
        include_str!("../src/ui/room.js"),
    ]
    .concat()
    .replace("&amp;", "&");
    for line in KO.lines().filter(|l| l.starts_with("  \"")) {
        let key: String = serde_json::from_str(&line[2..line.find("\": ").unwrap() + 1]).unwrap();
        let probe = key.trim_end();
        assert!(sources.contains(probe), "stale Korean entry: {key}");
    }
}
